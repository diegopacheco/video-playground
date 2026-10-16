use crate::AbrError;
use crate::algorithms::{self, catalog};
use crate::metrics::{Metrics, QoeWeights, evaluate};
use crate::player::{PlayerConfig, Session, simulate};
use crate::trace::{Trace, TraceSummary};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmRun {
    pub algorithm: String,
    pub name: String,
    pub metrics: Metrics,
    pub session: Session,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationReport {
    pub trace: TraceSummary,
    pub config: PlayerConfig,
    pub runs: Vec<AlgorithmRun>,
}

pub fn run(config: &PlayerConfig, trace: &Trace, algorithm_ids: &[String]) -> Result<SimulationReport, AbrError> {
    if algorithm_ids.is_empty() {
        return Err(AbrError::NoAlgorithms);
    }
    let top = config.ladder_kbps.last().copied().unwrap_or(0.0);
    let weights = QoeWeights::for_top_bitrate(top);
    let infos = catalog();
    let runs = algorithm_ids
        .iter()
        .map(|id| {
            let mut algorithm = algorithms::build(id)?;
            let session = simulate(config, trace, algorithm.as_mut())?;
            let name = infos.iter().find(|i| i.id == id).map(|i| i.name).unwrap_or(algorithm.id());
            Ok(AlgorithmRun {
                algorithm: id.clone(),
                name: name.to_string(),
                metrics: evaluate(&session, &weights),
                session,
            })
        })
        .collect::<Result<Vec<_>, AbrError>>()?;
    Ok(SimulationReport {
        trace: trace.summary(),
        config: config.clone(),
        runs,
    })
}

pub fn markdown_table(config: &PlayerConfig, traces: &[Trace]) -> Result<String, AbrError> {
    let ids: Vec<String> = catalog().iter().map(|i| i.id.to_string()).collect();
    let mut out = String::from(
        "| Trace | Algorithm | Avg bitrate (kbps) | Switches | Rebuffer (s) | Rebuffer events | Rebuffer ratio | Startup (s) | QoE |\n|---|---|---|---|---|---|---|---|---|\n",
    );
    for trace in traces {
        for r in run(config, trace, &ids)?.runs {
            let m = r.metrics;
            out.push_str(&format!(
                "| {} | {} | {:.0} | {} | {:.2} | {} | {:.2}% | {:.2} | {:.3} |\n",
                trace.name,
                r.name,
                m.avg_bitrate_kbps,
                m.switch_count,
                m.rebuffer_s,
                m.rebuffer_events,
                m.rebuffer_ratio * 100.0,
                m.startup_delay_s,
                m.qoe
            ));
        }
    }
    Ok(out)
}
