const { invoke } = window.__TAURI__.tauri;
let verif;
let folder1_js;
let folder2_js;

async function verify() {
  verif.innerHTML = "";
  folder1_js = document.querySelector("#folder1-input").value;
  folder2_js = document.querySelector("#folder2-input").value;
  let security = document.querySelector("#secure").checked;
  verif.innerHTML = await invoke("verify", {folder1: folder1_js, folder2: folder2_js, secure: security});
}

window.addEventListener("DOMContentLoaded", () => {
  verif = document.querySelector("#verify");

  document.querySelector("#verify-button").addEventListener("click", (e) => {
    e.preventDefault();
    verify();
  });
});
