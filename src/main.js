const { invoke } = window.__TAURI__.tauri;
let verif;
let folder1_js;
let folder2_js;

async function verify() {
  let loader = document.querySelector("#loader");
  verif.innerHTML = "";
  loader.classList.remove("hide-loader");
  folder1_js = document.querySelector("#folder1-input").value;
  folder2_js = document.querySelector("#folder2-input").value;
  let security = document.querySelector("#secure").checked;
  let json = await invoke("verify", {folder1: folder1_js, folder2: folder2_js, secure: security});
  loader.classList.add("hide-loader");
  let files_diff = json["different files"];
  if (files_diff.length == 0 && json["only folder1"].length == 0 && json["only folder2"].length == 0) {
    let stringResult = "All the files are identical and are present in each of the directories. ✅";
    verif.innerHTML = stringResult+"<br> time: "+json["time"]
  +"<br> number of files in folder 1 : " + json["Length of folder1"]
  +"<br> number of files in folder 2 : " + json["Length of folder2"]
  +"<br> excluded folders : " + json["excluded folders"].map((element) => "<br>"+element);
    return;
  }
  if (json["only folder1"].length == json["Length of folder1"] && json["only folder2"].length == json["Length of folder2"] ) {
    let stringResult = "All the files are different (perhaps one of the folders is included in the other?). ❌";
    verif.innerHTML = stringResult+"<br> time: "+json["time"]
    +"<br> number of files in folder 1 : " + json["Length of folder1"]
    +"<br> number of files in folder 2 : " + json["Length of folder2"]
    +"<br> excluded folders : " + json["excluded folders"].map((element) => "<br>"+element);
    return;
  }
  let stringResult = `<center><table> <tbody> 
      <tr> 
      <th scope="col">File</th>
      <th scope="col">In folder 1</th>
      <th scope="col">In folder 2</th>
      <th scope="col">Equals ?</th>
    </tr>`;
  let f1_files_set = new Set(json["f1_files"]);
  let f2_files_set = new Set(json["f2_files"]);
  let different_files = new Set(json["different files"])
  for (const file of json["all files"].sort()) {
    let i1 = f1_files_set.has(file);
    let i2 = f2_files_set.has(file);
    let i3 = different_files.has(file);
      stringResult += `<tr><th scope="row">${file}</th>`
      if (i1) {
        stringResult += `<td><code>✅</code></td>`;
      }
      else {
        stringResult += `<td><code>❌</code></td>`;
      }
      if (i2) {
        stringResult += `<td><code>✅</code></td>`;
      }
      else {
        stringResult += `<td><code>❌</code></td>`;
      }
      if (!i3 && i1 && i2) {
        stringResult += `<td><code>✅</code></td>`;
      }
      else if (i3 && i1 && i2) {
        stringResult += `<td><code>❌</code></td>`;
      }
      else {
        stringResult += `<td><code></code></td>`;
      }
      stringResult += `</tr>`;
  }
  verif.innerHTML = stringResult+`</table></center>`+"<br> time: "+json["time"]
  +"<br> number of files in folder 1 : " + json["Length of folder1"]
  +"<br> number of files in folder 2 : " + json["Length of folder2"]
  +"<br> excluded folders : " + json["excluded folders"].map((element) => "<br>"+element);
}

window.addEventListener("DOMContentLoaded", () => {
  verif = document.querySelector("#verify");

  document.querySelector("#verify-button").addEventListener("click", (e) => {
    e.preventDefault();
    verify();
  });
});
