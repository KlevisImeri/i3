use std::{env, error::Error, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Output {
    name: String,
    connected: bool,
}

impl Output {
    fn is_internal(&self) -> bool {
        ["eDP", "LVDS", "DSI"]
            .iter()
            .any(|prefix| self.name.starts_with(prefix))
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("i3-display: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    let dry_run = match arguments.as_slice() {
        [] => false,
        [flag] if flag == "--dry-run" => true,
        _ => return Err("usage: i3-display [--dry-run]".into()),
    };
    let wayland = env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");

    let outputs = if wayland {
        parse_sway(&query("swaymsg", &["-r", "-t", "get_outputs"])?)?
    } else {
        parse_xrandr(&query("xrandr", &["--query"])?)
    };
    let selected = select_output(&outputs)
        .ok_or("no connected monitors detected; leaving configuration unchanged")?;

    println!("Using {} and disabling all other outputs.", selected.name);
    if wayland {
        // Enable the selected screen before disabling any others.
        apply("swaymsg", &["output", &selected.name, "enable"], dry_run)?;
        for output in &outputs {
            if output.name != selected.name {
                apply("swaymsg", &["output", &output.name, "disable"], dry_run)?;
            }
        }
    } else {
        let arguments = xrandr_arguments(&outputs, selected);
        let arguments: Vec<_> = arguments.iter().map(String::as_str).collect();
        apply("xrandr", &arguments, dry_run)?;
    }
    Ok(())
}

fn select_output(outputs: &[Output]) -> Option<&Output> {
    outputs
        .iter()
        .find(|output| output.connected && !output.is_internal())
        .or_else(|| outputs.iter().find(|output| output.connected))
}

fn parse_xrandr(text: &str) -> Vec<Output> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            let connected = match fields.next()? {
                "connected" => true,
                "disconnected" => false,
                _ => return None,
            };
            Some(Output {
                name: name.into(),
                connected,
            })
        })
        .collect()
}

fn parse_sway(text: &str) -> Result<Vec<Output>> {
    let outputs: serde_json::Value = serde_json::from_str(text)?;
    let outputs = outputs
        .as_array()
        .ok_or("expected an array of Sway outputs")?;
    outputs
        .iter()
        .map(|output| {
            let name = output["name"]
                .as_str()
                .ok_or("Sway output is missing its name")?;
            // Disabled Sway outputs are still connected and can be enabled.
            Ok(Output {
                name: name.into(),
                connected: true,
            })
        })
        .collect()
}

fn xrandr_arguments(outputs: &[Output], selected: &Output) -> Vec<String> {
    let mut arguments: Vec<String> = [
        "--output",
        &selected.name,
        "--primary",
        "--auto",
        "--pos",
        "0x0",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    for output in outputs {
        if output.name != selected.name {
            arguments.extend(["--output".into(), output.name.clone(), "--off".into()]);
        }
    }
    arguments
}

fn query(program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program).args(arguments).output()?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn apply(program: &str, arguments: &[&str], dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{program} {}", arguments.join(" "));
        return Ok(());
    }

    // Sway can exit successfully while reporting a failed command in its JSON.
    let reply = query(program, arguments)?;
    if program == "swaymsg" {
        let results: serde_json::Value = serde_json::from_str(&reply)?;
        let results = results.as_array().ok_or("expected Sway command results")?;
        if results.is_empty() || results.iter().any(|result| result["success"] != true) {
            return Err(format!("Sway command failed: {reply}").into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unplugged_monitor_is_disabled() {
        let outputs = parse_xrandr(
            "Screen 0: current 2560 x 1440\neDP connected primary 1920x1080+0+0\nHDMI-A-0 disconnected 2560x1440+0+0\n   1920x1080 60.01*+",
        );
        assert_eq!(outputs.len(), 2);
        let selected = select_output(&outputs).unwrap();
        assert_eq!(selected.name, "eDP");
        assert_eq!(
            xrandr_arguments(&outputs, selected),
            [
                "--output",
                "eDP",
                "--primary",
                "--auto",
                "--pos",
                "0x0",
                "--output",
                "HDMI-A-0",
                "--off"
            ]
        );
    }

    #[test]
    fn external_monitor_is_preferred() {
        let outputs = parse_xrandr("eDP-1 connected\nDisplayPort-0 connected\nDP-2 connected");
        assert_eq!(select_output(&outputs).unwrap().name, "DisplayPort-0");
    }

    #[test]
    fn no_connected_outputs_means_no_selection() {
        let outputs = parse_xrandr("eDP disconnected\nHDMI-A-0 disconnected");
        assert!(select_output(&outputs).is_none());
        assert!(select_output(&[]).is_none());
    }

    #[test]
    fn sway_detects_disabled_internal_screen() {
        let outputs = parse_sway(r#"[{"name":"eDP-1","active":false}]"#).unwrap();
        assert_eq!(select_output(&outputs).unwrap().name, "eDP-1");
    }

    #[test]
    fn sway_prefers_external_screen() {
        let outputs = parse_sway(r#"[{"name":"eDP-1"},{"name":"DP-1"}]"#).unwrap();
        assert_eq!(select_output(&outputs).unwrap().name, "DP-1");
    }

    #[test]
    fn malformed_sway_output_is_rejected() {
        for text in ["not json", "{}", "[{}]"] {
            assert!(parse_sway(text).is_err());
        }
    }
}
