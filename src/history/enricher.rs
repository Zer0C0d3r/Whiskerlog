use super::Command;
use crate::history::detector::*;

pub struct CommandEnricher {
    host_detector: HostDetector,
    network_detector: NetworkDetector,
    package_detector: PackageDetector,
    danger_detector: DangerDetector,
    experiment_detector: ExperimentDetector,
}

impl Default for CommandEnricher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandEnricher {
    pub fn new() -> Self {
        Self {
            host_detector: HostDetector::new(),
            network_detector: NetworkDetector::new(),
            package_detector: PackageDetector::new(),
            danger_detector: DangerDetector::new(),
            experiment_detector: ExperimentDetector::new(),
        }
    }

    pub async fn enrich(&self, mut command: Command) -> Command {
        // Detect host context (ssh, docker, k8s)
        command.host_id = self.host_detector.detect(&command.command);

        // Detect network endpoints
        command.network_endpoints = self.network_detector.detect(&command.command);

        // Detect package operations
        command.packages_used = self.package_detector.detect(&command.command);

        // Assess danger level
        let danger_result = self.danger_detector.assess(&command.command);
        command.is_dangerous = danger_result.is_dangerous;
        command.danger_score = danger_result.score;
        command.danger_reasons = danger_result.reasons;

        // Detect if this is an experimental/learning command
        let experiment_result = self.experiment_detector.detect(&command.command);
        command.is_experiment = experiment_result.is_experiment;
        command.experiment_tags = experiment_result.tags;

        command
    }
}
