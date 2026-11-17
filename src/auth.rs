use gloo_net::http::Request;
use leptos::*;
use serde::Deserialize;
#[derive(Clone, Deserialize)]
struct Person {
    name: String,
    nim: String,
    group_name: String,
    role: String,
}
#[component]
pub fn LoginGate() -> impl IntoView {
    let (person, set_person) = create_signal(None::<Person>);
    let (nim, set_nim) = create_signal(String::new());
    let (busy, set_busy) = create_signal(true);
    let (error, set_error) = create_signal(String::new());
    spawn_local(async move {
        if let Ok(r) = Request::get("/api/me").send().await {
            if r.ok() {
                if let Ok(p) = r.json::<Person>().await {
                    set_person.set(Some(p));
                }
            }
        }
        set_busy.set(false);
    });
    view! {
    {move || match person.get(){Some(p)=>view!{
    <div class="session-bar"><span>{format!("{} · {} · Kelompok {}{}",p.name,p.nim,p.group_name,if p.role=="test"{" · AKUN UJI"}else{""})}</span><button on:click=move |_| spawn_local(async move{if let Ok(r)=Request::post("/api/logout").send().await{if r.ok(){set_person.set(None);set_nim.set(String::new());}}})>"Keluar"</button></div>
    <crate::App/>
    }.into_view(),None=>view!{
    <main class="login-page"><p class="course">"Teknik Industri · Universitas Muhammadiyah Malang"</p><h1>"Masuk praktikum"</h1><p>"Perilaku Organisasi"</p>
    <form on:submit=move |e|{e.prevent_default();if busy.get(){return;}set_busy.set(true);set_error.set(String::new());let value=nim.get();spawn_local(async move{
    let result=async {let req=Request::post("/api/login").json(&serde_json::json!({"nim":value.trim()})).map_err(|_|"Permintaan tidak valid.".to_string())?;let r=req.send().await.map_err(|_|"Tidak dapat menghubungi server.".to_string())?;
    if r.ok(){r.json::<Person>().await.map_err(|_|"Respons server tidak valid.".to_string())}else{let v=r.json::<serde_json::Value>().await.unwrap_or_default();Err(v["error"].as_str().unwrap_or("Login gagal.").to_string())}}.await;
    match result{Ok(p)=>set_person.set(Some(p)),Err(e)=>set_error.set(e)}set_busy.set(false);
    });}>
    <label for="nim">"NIM"</label><input id="nim" name="nim" type="text" inputmode="numeric" autocomplete="username" maxlength="25" required prop:value=nim on:input=move |e|set_nim.set(event_target_value(&e))/><button class="primary" type="submit" disabled=busy>{move||if busy.get(){"Memproses…"}else{"Masuk"}}</button>
    <p role="alert">{error}</p></form><p class="note">"Hanya NIM yang terdaftar. Gunakan NIM sendiri. Login NIM tidak memverifikasi identitas pemiliknya."</p></main>
    }.into_view()}}
    }
}
#[component]
pub fn ReceiptControls(upload: StoredValue<Option<(String, Vec<u8>)>>) -> impl IntoView {
    let (consent, set_consent) = create_signal(false);
    let (busy, set_busy) = create_signal(false);
    let (message, set_message) = create_signal(String::new());
    let (link, set_link) = create_signal(None::<String>);
    view! {<section class="research"><h2>"Struk terverifikasi untuk aslab"</h2><p>"Server menghitung ulang file asli yang terakhir diunggah. Dokumen dikirim sementara dan tidak disimpan; identitas, nama file, hash, waktu, dan hasil disimpan untuk verifikasi. Jika teks pratinjau diedit, unggah ulang file sebelum menerbitkan struk."</p>
    <label><input type="checkbox" prop:checked=consent on:change=move |e|set_consent.set(event_target_checked(&e))/ >" Saya setuju file dikirim sementara untuk pemeriksaan server dan catatan hasil disimpan."</label>
    <button class="primary" disabled=move||busy.get()||!consent.get() on:click=move |_|{
    set_link.set(None);let Some((name,bytes))=upload.get_value()else{set_message.set("Unggah DOCX/TXT terlebih dahulu. Teks tempel tidak memiliki file asli untuk struk.".into());return;};
    set_busy.set(true);set_message.set("Menghitung ulang dan menerbitkan struk…".into());
    spawn_local(async move {
    let result=async{let endpoint=format!("/api/receipts?name={}",js_sys::encode_uri_component(&name));let req=Request::post(&endpoint).header("content-type","application/octet-stream").header("x-consent","temporary-processing").body(bytes).map_err(|_|"File tidak dapat dikirim.".to_string())?;let r=req.send().await.map_err(|_|"Server tidak dapat dihubungi. Periksa koneksi atau sesi login.".to_string())?;let v=r.json::<serde_json::Value>().await.map_err(|_|"Respons tidak valid.".to_string())?;if r.ok(){Ok(v["receipt_url"].as_str().ok_or("Struk tidak tersedia.")?.to_string())}else{Err(v["error"].as_str().unwrap_or("Struk gagal diterbitkan.").to_string())}}.await;
    match result{Ok(url)=>{set_link.set(Some(url));set_message.set("Struk tersimpan. Buka struk, lalu screenshot untuk aslab.".into());},Err(e)=>set_message.set(e)}set_busy.set(false);
    });
    }>{move||if busy.get(){"Memproses…"}else{"Terbitkan struk"}}</button><p role="status">{message}</p>{move||link.get().map(|url|view!{<a class="receipt-link" href=url target="_blank" rel="noopener">"Buka struk untuk screenshot"</a>})}</section>}
}
