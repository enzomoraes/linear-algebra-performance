use regex::Regex;
use std::fs;

#[derive(Debug)]
struct Metric {
    name: String,
    value: f64,
    var: f64,    // var% relativo do perf
    stddev: f64, // desvio padrão calculado
    share: String,
}

fn colorize_var(var: f64) -> String {
    if var < 5.0 {
        format!("\x1b[32m{:.2}%\x1b[0m", var) // verde
    } else if var < 15.0 {
        format!("\x1b[33m{:.2}%\x1b[0m", var) // amarelo
    } else {
        format!("\x1b[31m{:.2}%\x1b[0m", var) // vermelho
    }
}

fn main() {
    let content = fs::read_to_string("test_results.txt").expect("Error reading file");

    let blocks: Vec<&str> = content.split("# TEST_NAME:").skip(1).collect();

    let re_name = Regex::new(r"(?m)^([^\n]+)").unwrap();

    let re_metric = Regex::new(
        r"^\s*(?P<value>[0-9\.,]+|<not counted>)\s+(?P<name>[a-zA-Z0-9_/.\-]+)\/?\s*(?:#.*)?\s*\(\s*\+-\s*(?P<var>[0-9\.,]+)%\s*\)?(?:\s*\(\s*(?P<share>[0-9\.,]+)%\s*\))?"
    ).unwrap();

    let re_task_clock =
        Regex::new(r"(?P<value>[0-9\.,]+)\s+msec task-clock.*\(\s*\+-\s*(?P<var>[0-9\.,]+)%\s*\)")
            .unwrap();

    let re_time_elapsed = Regex::new(
        r"(?P<value>[0-9\.,]+)\s*\+\-\s*(?P<stderr>[0-9\.,]+)\s*seconds time elapsed(?:\s*\(\s*\+-\s*(?P<var>[0-9\.,]+)%\s*\))?"
    ).unwrap();

    for block in blocks {
        let name_caps = re_name.captures(block).unwrap();
        let test_name = name_caps[1].trim();

        println!("\n================ {} ================", test_name);

        let mut metrics = Vec::new();
        let mut time_elapsed: Option<(f64, f64, f64)> = None;

        for line in block.lines() {
            if let Some(c) = re_metric.captures(line) {
                let value = if &c["value"] == "<not counted>" {
                    0.0
                } else {
                    c["value"]
                        .replace(".", "")
                        .replace(",", ".")
                        .parse::<f64>()
                        .unwrap_or(0.0)
                };

                let var = c["var"].replace(",", ".").parse::<f64>().unwrap_or(0.0);

                // Desvio padrão calculado
                let stddev = value * (var / 100.0);

                let share = c
                    .name("share")
                    .map(|m| m.as_str().replace(",", "."))
                    .unwrap_or("--".to_string());

                metrics.push(Metric {
                    name: c["name"].to_string(),
                    value,
                    var,
                    stddev,
                    share,
                });
            } else if let Some(c) = re_task_clock.captures(line) {
                let value = c["value"]
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f64>()
                    .unwrap();

                let var = c["var"].replace(",", ".").parse::<f64>().unwrap();

                let stddev = value * (var / 100.0);

                metrics.push(Metric {
                    name: "task-clock (msec)".to_string(),
                    value,
                    var,
                    stddev,
                    share: "--".to_string(),
                });
            } else if let Some(c) = re_time_elapsed.captures(line) {
                let val = c["value"].replace(",", ".").parse::<f64>().unwrap();
                let stderr = c["stderr"].replace(",", ".").parse::<f64>().unwrap();
                let var = c
                    .name("var")
                    .map(|m| m.as_str().replace(",", ".").parse::<f64>().unwrap())
                    .unwrap_or(0.0);

                time_elapsed = Some((val, stderr, var));
            }
        }

        // Header novo
        println!("Métrica                           |     Valor |   Var%  ratio of std-dev/mean |    STD | Share");
        println!("----------------------------------------------------------------------------");

        for m in metrics {
            let var_colored = colorize_var(m.var);

            println!(
                "{:<32} | {:>10.0} | {:>7} | {:>7.0} | {:>6}",
                m.name, m.value, var_colored, m.stddev, m.share
            );
        }

        if let Some((val, stderr, var)) = time_elapsed {
            let var_colored = colorize_var(var);
            let stddev = val * (var / 100.0);

            println!(
                "\nTime elapsed: {:.6} s (± {:.6} s, Var% = {}, STD = {:.6} s)",
                val, stderr, var_colored, stddev
            );
        } else {
            println!("\nTime elapsed: NÃO ENCONTRADO");
        }
    }
}
