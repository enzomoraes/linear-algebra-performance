use regex::Regex;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::fmt;
use std::env;

// Estrutura para armazenar os dados de cada teste
#[derive(Default, Debug)]
struct TestMetrics {
    name: String,
    abbreviated_name: String,
    task_clock: String,
    cpus_utilized: String,
    task_clock_cv: String,      // Variância percentual
    cache_misses: String,
    cache_misses_cv: String,
    branch_misses: String,
    branch_misses_cv: String,
    instructions: String,
    instructions_cv: String,
    time_elapsed: String,
    time_var_nominal: String,   // +- 0,0219
    time_var_percent: String,   // +- 1,63%
}

impl TestMetrics {
    fn new() -> Self {
        Self { ..Default::default() }
    }
}

// Implementação para formatar como linha de CSV
impl fmt::Display for TestMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.name,
            self.abbreviated_name,
            self.task_clock,
            self.cpus_utilized,
            self.task_clock_cv,
            self.cache_misses,
            self.cache_misses_cv,
            self.branch_misses,
            self.branch_misses_cv,
            self.instructions,
            self.instructions_cv,
            self.time_elapsed,
            self.time_var_nominal,
            self.time_var_percent
        )
    }
}

fn main() -> io::Result<()> {
    // Lê argumentos da linha de comando
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Uso: {} <caminho_do_arquivo_de_entrada>", args[0]);
        eprintln!("Exemplo: {} test_results.txt", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    
    // Gera o nome do arquivo de saída baseado no nome do arquivo de entrada
    let input_stem = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("results");
    let output_path = format!("{}_results.csv", input_stem);

    // Verifica se o arquivo de entrada existe antes de tentar ler
    if !Path::new(input_path).exists() {
        eprintln!("Erro: O arquivo '{}' não foi encontrado.", input_path);
        std::process::exit(1);
    }

    // Lê o arquivo inteiro para uma String
    let content = std::fs::read_to_string(input_path)?;

    // Compilação das Regex
    let re_test_name = Regex::new(r"^# TEST_NAME:\s+(.+)$").unwrap();
    
    // Regex para task-clock: captura msec, cpus e variância
    let re_task_clock = Regex::new(r"^\s*([\d\.,]+)\s+msec task-clock\s+#\s+([\d,]+)\s+CPUs utilized\s+\(\s+\+-\s+([\d,]+)%\s+\)").unwrap();

    // Regex para métricas do core: captura valor, nome da métrica e variância
    let re_core_metrics = Regex::new(r"^\s*([\d\.]+)\s+cpu_core/(cache-misses|branch-misses|instructions)/\s+\(\s+\+-\s+([\d,]+)%\s+\)").unwrap();

    // Regex para tempo decorrido: captura tempo, var nominal e var percentual
    let re_elapsed = Regex::new(r"^\s*([\d,]+)\s+\+-\s+([\d,]+)\s+seconds time elapsed\s+\(\s+\+-\s+([\d,]+)%\s+\)").unwrap();

    let mut metrics = Vec::new();
    let mut current_metric = TestMetrics::new();
    let mut collecting = false;

    // Itera linha a linha do arquivo lido
    for line in content.lines() {
        let line = line.trim();

        if let Some(caps) = re_test_name.captures(line) {
            if collecting {
                metrics.push(current_metric);
            }
            current_metric = TestMetrics::new();
            current_metric.name = caps[1].to_string();
            current_metric.abbreviated_name = match current_metric.name.as_str() {
                "naive_fragmented" => "NF".to_string(),
                "contiguous_strided" => "CS".to_string(),
                "contiguous_parallel_strided" => "CPS".to_string(),
                "contiguous_tiled" => "CT".to_string(),
                "contiguous_parallel_tiled" => "CPT".to_string(),
                _ => current_metric.name.clone(),
            };
            collecting = true;
            continue;
        }

        if !collecting {
            continue;
        }

        if let Some(caps) = re_task_clock.captures(line) {
            current_metric.task_clock = caps[1].to_string();
            current_metric.cpus_utilized = caps[2].to_string();
            current_metric.task_clock_cv = caps[3].to_string();
        } else if let Some(caps) = re_core_metrics.captures(line) {
            let value = caps[1].to_string();
            let metric_type = &caps[2];
            let cv = caps[3].to_string();

            match metric_type {
                "cache-misses" => {
                    current_metric.cache_misses = value;
                    current_metric.cache_misses_cv = cv;
                },
                "branch-misses" => {
                    current_metric.branch_misses = value;
                    current_metric.branch_misses_cv = cv;
                },
                "instructions" => {
                    current_metric.instructions = value;
                    current_metric.instructions_cv = cv;
                },
                _ => {}
            }
        } else if let Some(caps) = re_elapsed.captures(line) {
            current_metric.time_elapsed = caps[1].to_string();
            current_metric.time_var_nominal = caps[2].to_string();
            current_metric.time_var_percent = caps[3].to_string();
        }
    }

    if collecting {
        metrics.push(current_metric);
    }

    // Cria e escreve no arquivo CSV
    let mut file = File::create(&output_path)?;
    
    // Escreve cabeçalho
    writeln!(file, "Test Name;Abbreviated Name;Task Clock (msec);CPUs Utilized;Task Clock CV(%);Core Cache Misses;Cache Miss CV(%);Core Branch Misses;Branch Miss CV(%);Core Instructions;Instructions CV(%);Time Elapsed (s);Time Var Nominal (+-);Time Var Percent (+-%)")?;

    // Escreve linhas
    let num_records = metrics.len();
    for m in metrics {
        writeln!(file, "{}", m)?;
    }

    println!("Sucesso! Arquivo '{}' gerado com {} registros.", output_path, num_records);

    Ok(())
}