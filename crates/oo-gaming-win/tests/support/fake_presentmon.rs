//! PresentMon falso, só para testar o encanamento de `FpsCapture` sem admin nem jogo.
//! Recebe os mesmos argumentos do real. `--process_name game.exe`: imprime 80 s de frames de
//! 10 ms e sai. `--process_name hang.exe`: imprime e fica parado (testa o encerramento forçado).
//! `--terminate_existing_session`: sai na hora.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--terminate_existing_session") {
        return;
    }
    let game = args
        .iter()
        .position(|a| a == "--process_name")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_default();
    println!("Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,Dropped,TimeInSeconds,msInPresentAPI,msBetweenPresents");
    let mut t = 0.0f64;
    for _ in 0..8000 {
        t += 0.01;
        println!("{game},42,0x0000000000000001,DXGI,0,0,0,{t:.4},0.1,10.0");
    }
    if game == "hang.exe" {
        std::thread::sleep(std::time::Duration::from_secs(120));
    }
}
