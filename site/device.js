// The 3D Sprig in the hero. Loads Hack Club's open-source sprig.glb (MIT,
// github.com/hackclub/sprig) and renders the Locket firmware UI live onto the
// device's screen mesh — the same screen-texture trick the Sprig site uses.
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { DRACOLoader } from "three/addons/loaders/DRACOLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";

const mount = document.getElementById("sprig3d");
if (mount) {
  const W = () => mount.clientWidth || 380;
  const H = () => mount.clientHeight || 360;

  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: "low-power" });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.setSize(W(), H());
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  mount.appendChild(renderer.domElement);

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, W() / H(), 0.1, 100);
  camera.position.set(0, 0, 0.6);

  const pmrem = new THREE.PMREMGenerator(renderer);
  scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
  scene.add(new THREE.HemisphereLight(0xffffff, 0x202020, 1.1));
  const key = new THREE.DirectionalLight(0xffffff, 1.4);
  key.position.set(1, 2, 3);
  scene.add(key);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableZoom = false;
  controls.enablePan = false;
  controls.enableDamping = true;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 1.4;
  // stop the gentle intro spin the moment the user grabs it, so it doesn't fight them
  controls.addEventListener("start", () => { controls.autoRotate = false; });

  // --- the screen: a canvas we draw the Locket UI onto, used as a texture ---
  const cv = document.createElement("canvas");
  cv.width = 320; cv.height = 256; // 160x128, doubled
  const ctx = cv.getContext("2d");
  const tex = new THREE.CanvasTexture(cv);
  tex.flipY = false;
  tex.magFilter = THREE.NearestFilter;
  tex.minFilter = THREE.NearestFilter;
  tex.colorSpace = THREE.SRGBColorSpace;

  const SCENES = ["boot", "compose", "reply"];
  let si = 0;

  function vars() {
    const cs = getComputedStyle(document.documentElement);
    const v = (n) => cs.getPropertyValue(n).trim();
    return {
      bg: v("--bg"), text: v("--text"), dim: v("--dim"),
      surface: v("--surface"), st: v("--surface-text"),
      accent: v("--accent"), onAccent: v("--on-accent"),
    };
  }

  function draw() {
    const t = vars(), sc = SCENES[si];
    ctx.fillStyle = t.bg; ctx.fillRect(0, 0, 320, 256);
    ctx.textBaseline = "top"; ctx.textAlign = "left";

    if (sc === "boot") {
      ctx.textAlign = "center";
      ctx.fillStyle = t.accent; ctx.font = "bold 52px monospace";
      ctx.fillText("Locket", 160, 64);
      ctx.fillStyle = t.text; ctx.font = "18px monospace";
      ctx.fillText("v0.1.0 - pocket AI", 160, 132);
      ctx.fillStyle = t.dim; ctx.fillText("Connecting WiFi...", 160, 168);
      return tex.needsUpdate = true;
    }

    // header bar
    ctx.fillStyle = t.surface; ctx.fillRect(0, 0, 320, 32);
    ctx.fillStyle = t.st; ctx.font = "bold 18px monospace";
    ctx.fillText(sc === "reply" ? "CHAT" : "AI: Default", 10, 8);
    for (let i = 0; i < 4; i++) { ctx.fillRect(276 + i * 10, 24 - (i + 1) * 4, 6, (i + 1) * 4); }

    if (sc === "compose") {
      ctx.strokeStyle = t.dim; ctx.lineWidth = 2; ctx.strokeRect(10, 44, 300, 52);
      ctx.fillStyle = t.text; ctx.font = "20px monospace"; ctx.fillText("explain wifi", 20, 58);
      ctx.fillStyle = t.accent; ctx.fillRect(10, 112, 24, 26);
      ctx.fillStyle = t.onAccent; ctx.font = "bold 18px monospace"; ctx.fillText("L", 16, 116);
      ctx.fillStyle = t.text; ctx.font = "18px monospace"; ctx.fillText("simply", 44, 116);
      ctx.fillStyle = t.dim; ctx.font = "15px monospace";
      ctx.fillText("Wabcd Aefgh Sijkl", 10, 206);
      ctx.fillText("Iqrst Juvwx Kyz.,", 10, 228);
    } else {
      ctx.fillStyle = t.text; ctx.font = "19px monospace";
      [">> WiFi is your device", "chatting with a router", "by radio - a walkie-", "talkie for data"]
        .forEach((l, i) => ctx.fillText(l, 12, 48 + i * 28));
    }
    tex.needsUpdate = true;
  }

  // --- load the model ---
  const draco = new DRACOLoader();
  draco.setDecoderPath("https://cdn.jsdelivr.net/npm/three@0.183.2/examples/jsm/libs/draco/gltf/");
  const loader = new GLTFLoader();
  loader.setDRACOLoader(draco);

  loader.load(
    "sprig.glb",
    (gltf) => {
      const model = gltf.scene;
      // Orient FIRST (stand the board upright, LCD toward the camera), THEN
      // recenter — so the geometric centre lands exactly on the orbit target.
      // (Rotating after centering is what pushed the pivot off to one side.)
      model.rotation.x = Math.PI / 2;
      scene.add(model);
      const box = new THREE.Box3().setFromObject(model);
      const sphere = box.getBoundingSphere(new THREE.Sphere());
      model.position.sub(sphere.center);
      const r = sphere.radius || 1;
      const fovr = (camera.fov * Math.PI) / 180;
      const dist = (r / Math.sin(fovr / 2)) * 1.05;
      camera.position.set(0, 0, dist); // straight-on: screen front and centre
      camera.near = dist / 100;
      camera.far = dist * 100;
      camera.updateProjectionMatrix();
      controls.target.set(0, 0, 0);
      controls.update();

      // find the screen mesh and put our canvas on its "Glow Glass" material
      let glass = null;
      model.traverse((o) => {
        if (o.material && o.material.name === "Glow Glass") glass = o;
      });
      if (glass) {
        glass.material = new THREE.MeshBasicMaterial({ map: tex, toneMapped: false });
      }
      draw();
      const hint = document.querySelector(".model-hint");
      if (hint) hint.style.opacity = "1";
    },
    undefined,
    (err) => { console.error("sprig.glb failed:", err); }
  );

  // --- loop: spin + cycle the screen scenes ---
  let last = 0;
  function frame(t) {
    requestAnimationFrame(frame);
    controls.update();
    if (t - last > 2600) { last = t; si = (si + 1) % SCENES.length; draw(); }
    renderer.render(scene, camera);
  }
  requestAnimationFrame(frame);

  window.addEventListener("resize", () => {
    renderer.setSize(W(), H());
    camera.aspect = W() / H();
    camera.updateProjectionMatrix();
  });

  // redraw the screen in the new palette whenever the site theme changes
  new MutationObserver(draw).observe(document.documentElement, {
    attributes: true, attributeFilter: ["data-theme"],
  });
}
