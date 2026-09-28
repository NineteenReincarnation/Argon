use serde::Serialize;
use sysinfo::{Pid, System};

const AUTO_SELECT_MIN_SCORE: i32 = 60;
const AUTO_SELECT_MARGIN: i32 = 20;

#[derive(Debug, Clone, Serialize)]
pub struct TargetProcess {
    pub pid: u32,
    pub name: String,
    pub executable_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub target: TargetProcess,
    pub score: i32,
    pub reasons: Vec<&'static str>,
}

pub fn discover_minecraft_candidates() -> Vec<Candidate> {
    let system = System::new_all();
    let mut candidates = system
        .processes()
        .values()
        .filter_map(candidate_from_process)
        .filter(|candidate| candidate.score > 0)
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.target.pid.cmp(&right.target.pid))
    });
    candidates
}

pub fn target_by_pid(pid: u32) -> Result<TargetProcess, String> {
    let system = System::new_all();
    let process = system
        .process(Pid::from_u32(pid))
        .ok_or_else(|| format!("process {pid} does not exist or is not visible"))?;

    Ok(target_from_process(process))
}

pub fn select_target(explicit_pid: Option<u32>) -> Result<TargetProcess, String> {
    if let Some(pid) = explicit_pid {
        return target_by_pid(pid);
    }

    let candidates = discover_minecraft_candidates();
    let first = candidates.first().ok_or_else(|| {
        "no likely Minecraft Java client process was found; use 'list' or pass --pid".to_owned()
    })?;

    if first.score < AUTO_SELECT_MIN_SCORE {
        return Err(format!(
            "no process reached the automatic Minecraft confidence threshold (best PID {} score {}); pass --pid explicitly",
            first.target.pid, first.score
        ));
    }

    if let Some(second) = candidates.get(1)
        && first.score - second.score < AUTO_SELECT_MARGIN
    {
        return Err(format!(
            "multiple plausible Minecraft processes were found (PIDs {} and {}); pass --pid explicitly",
            first.target.pid, second.target.pid
        ));
    }

    Ok(first.target.clone())
}

fn candidate_from_process(process: &sysinfo::Process) -> Option<Candidate> {
    let target = target_from_process(process);
    let command_line = process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    let (score, reasons) = score_process(&target.name, &command_line);
    (score > 0).then_some(Candidate {
        target,
        score,
        reasons,
    })
}

fn target_from_process(process: &sysinfo::Process) -> TargetProcess {
    TargetProcess {
        pid: process.pid().as_u32(),
        name: process.name().to_string_lossy().into_owned(),
        executable_name: process
            .exe()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned()),
    }
}

fn score_process(name: &str, command_line: &str) -> (i32, Vec<&'static str>) {
    let name = name.to_ascii_lowercase();
    let command_line = command_line.to_ascii_lowercase();

    let mut score = 0;
    let mut reasons = Vec::new();

    if name == "javaw.exe" || name == "java.exe" || name == "javaw" || name == "java" {
        score += 10;
        reasons.push("java process");
    }

    if command_line.contains("net.fabricmc.loader.impl.launch.knot.knotclient") {
        score += 100;
        reasons.push("Fabric KnotClient");
    }

    if command_line.contains("net.minecraft.client.main.main") {
        score += 80;
        reasons.push("Minecraft client main");
    }

    if command_line.contains("fabric-loader") || command_line.contains("fabricloader") {
        score += 25;
        reasons.push("Fabric loader");
    }

    if command_line.contains("minecraft") {
        score += 15;
        reasons.push("Minecraft command line");
    }

    if command_line.contains("org.gradle") || command_line.contains("gradle") {
        score -= 100;
        reasons.push("Gradle process");
    }

    (score, reasons)
}

#[cfg(test)]
mod tests {
    use super::score_process;

    #[test]
    fn fabric_client_outranks_generic_java() {
        let (fabric, _) = score_process(
            "javaw.exe",
            "net.fabricmc.loader.impl.launch.knot.KnotClient --gameDir C:/game",
        );
        let (generic, _) = score_process("javaw.exe", "-jar unrelated.jar");

        assert!(fabric > generic);
        assert!(fabric >= 100);
    }

    #[test]
    fn gradle_is_not_selected_as_minecraft() {
        let (score, _) = score_process(
            "java.exe",
            "org.gradle.launcher.daemon.bootstrap.GradleDaemon minecraft",
        );

        assert!(score < 0);
    }
}
