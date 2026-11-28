use oo_fps::{append_history, read_history, CsvReader, FpsRecord, FrameStats, WARMUP_S};

const HEADER: &str = "Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,Dropped,TimeInSeconds,msInPresentAPI,msBetweenPresents,AllowsTearing";

fn row(sc: &str, t: f64, ms: f64) -> String {
    format!("game.exe,1234,{sc},DXGI,0,0,0,{t},0.1,{ms},1")
}

/// Alimenta `n` frames de `ms` cada na swap chain `sc`, a partir de `t0`.
fn feed(r: &mut CsvReader, s: &mut FrameStats, sc: &str, t0: f64, n: usize, ms: f64) -> f64 {
    let mut t = t0;
    for _ in 0..n {
        t += ms / 1000.0;
        let f = r.line(&row(sc, t, ms)).expect("linha válida");
        s.push(&f);
    }
    t
}

#[test]
fn cabecalho_por_nome_e_linha_invalida() {
    let mut r = CsvReader::new();
    assert!(r.line(HEADER).is_none(), "cabeçalho não é frame");
    let f = r.line(&row("0x01", 1.5, 16.6)).unwrap();
    assert_eq!(f.swapchain, "0x01");
    assert_eq!(f.runtime, "DXGI");
    assert!((f.t_s - 1.5).abs() < 1e-9 && (f.ms - 16.6).abs() < 1e-9);
    assert!(r.line("lixo,sem,colunas").is_none());
    assert!(r.line("").is_none());
}

#[test]
fn colunas_fora_de_ordem() {
    let mut r = CsvReader::new();
    r.line("msBetweenPresents,TimeInSeconds,Runtime,SwapChainAddress");
    let f = r.line("8.0,20.0,VULKAN,0xAB").unwrap();
    assert_eq!(
        (f.swapchain.as_str(), f.runtime.as_str()),
        ("0xAB", "VULKAN")
    );
    assert!((f.ms - 8.0).abs() < 1e-9 && (f.t_s - 20.0).abs() < 1e-9);
}

#[test]
fn sem_cabecalho_conhecido_nada_e_frame() {
    let mut r = CsvReader::new();
    r.line("a,b,c");
    assert!(r.line("1,2,3").is_none());
}

#[test]
fn aquecimento_e_percentis() {
    let mut r = CsvReader::new();
    let mut s = FrameStats::new(WARMUP_S);
    r.line(HEADER);
    // 15 s de carregamento a 100 ms/frame: tem que ser ignorado.
    feed(&mut r, &mut s, "0x1", 0.0, 140, 100.0);
    // Depois (a partir de 20 s, longe da borda): 980 frames de 10 ms e 20 de 40 ms.
    let t = feed(&mut r, &mut s, "0x1", 20.0, 980, 10.0);
    feed(&mut r, &mut s, "0x1", t, 20, 40.0);
    let m = s.summary().unwrap();
    assert_eq!(m.frames, 1000);
    assert!((m.measured_s - 10.6).abs() < 0.01, "{}", m.measured_s);
    assert!((m.avg_fps - 1000.0 / 10.6).abs() < 0.1, "{}", m.avg_fps);
    assert!((m.ft_p50_ms - 10.0).abs() <= 0.05, "{}", m.ft_p50_ms);
    assert!((m.ft_p99_ms - 40.0).abs() <= 0.05, "{}", m.ft_p99_ms);
    assert!((m.low1_fps - 25.0).abs() < 0.1, "{}", m.low1_fps);
    assert!((m.low01_fps - 25.0).abs() < 0.1, "{}", m.low01_fps);
    assert_eq!(m.runtime, "DXGI");
}

#[test]
fn vale_a_swap_chain_com_mais_frames() {
    let mut r = CsvReader::new();
    let mut s = FrameStats::new(0.0);
    r.line(HEADER);
    // Linhas em ordem cronológica, como o PresentMon escreve.
    let t = feed(&mut r, &mut s, "0xLAUNCHER", 0.0, 50, 33.3);
    feed(&mut r, &mut s, "0xGAME", t, 500, 5.0);
    let m = s.summary().unwrap();
    assert_eq!(m.frames, 500);
    assert!((m.avg_fps - 200.0).abs() < 0.5);
}

#[test]
fn sem_frames_nao_ha_sumario() {
    assert!(FrameStats::new(WARMUP_S).summary().is_none());
}

#[test]
fn sessao_curta_nao_e_valida() {
    let mut r = CsvReader::new();
    let mut s = FrameStats::new(0.0);
    r.line(HEADER);
    feed(&mut r, &mut s, "0x1", 0.0, 3000, 10.0); // 30 s
    let curta = FpsRecord::new("game.exe", 1000, 45, s.summary().unwrap());
    assert!(!curta.valid);
    feed(&mut r, &mut s, "0x1", 30.0, 4000, 10.0); // +40 s = 70 s
    let longa = FpsRecord::new("game.exe", 1000, 90, s.summary().unwrap());
    assert!(longa.valid);
}

#[test]
fn historico_jsonl_com_poda_e_linha_corrompida() {
    let dir = std::env::temp_dir().join(format!("oo-fps-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fps-history.jsonl");
    let mut r = CsvReader::new();
    let mut s = FrameStats::new(0.0);
    r.line(HEADER);
    feed(&mut r, &mut s, "0x1", 0.0, 100, 10.0);
    let sum = s.summary().unwrap();
    for i in 0..601u64 {
        append_history(&path, &FpsRecord::new("game.exe", i, 60, sum.clone())).unwrap();
    }
    let all = read_history(&path);
    assert_eq!(all.len(), 500, "podado para as 500 mais recentes");
    assert_eq!(all.first().unwrap().started, 101);
    assert_eq!(all.last().unwrap().started, 600);
    // Linha corrompida no meio não derruba a leitura.
    let mut txt = std::fs::read_to_string(&path).unwrap();
    txt.push_str("{corrompido\n");
    std::fs::write(&path, txt).unwrap();
    assert_eq!(read_history(&path).len(), 500);
    assert!(read_history(&dir.join("nao-existe.jsonl")).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
