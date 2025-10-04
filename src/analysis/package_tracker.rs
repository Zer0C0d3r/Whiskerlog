use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::history::Command;

#[derive(Debug, Clone)]
pub struct PackageAnalysis {
    pub total_package_operations: usize,
    pub managers_used: Vec<ManagerStats>,
    pub package_trends: Vec<PackageTrend>,
    pub version_conflicts: Vec<VersionConflict>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ManagerStats {
    pub manager: String,
    pub total_operations: usize,
    pub installs: usize,
    pub removes: usize,
    pub updates: usize,
    pub top_packages: Vec<PackageStats>,
}

#[derive(Debug, Clone)]
pub struct PackageStats {
    pub name: String,
    pub install_count: usize,
    pub remove_count: usize,
    pub first_installed: Option<DateTime<Utc>>,
    pub last_used: DateTime<Utc>,
    pub versions_seen: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PackageTrend {
    pub package: String,
    pub manager: String,
    pub trend_type: TrendType,
    pub frequency: usize,
    pub time_span_days: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrendType {
    FrequentInstalls,
    RepeatedInstalls, // Same package installed multiple times
    QuickRemoval,     // Installed then quickly removed
    VersionChurn,     // Multiple versions installed
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VersionConflict {
    pub package: String,
    pub manager: String,
    #[allow(dead_code)]
    pub versions: Vec<String>,
    pub conflict_type: ConflictType,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConflictType {
    MultipleVersions,
    DowngradeDetected,
    InconsistentVersioning,
}

pub struct PackageTracker;

impl Default for PackageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_package_usage(&self, commands: &[Command]) -> PackageAnalysis {
        let package_commands: Vec<_> = commands
            .iter()
            .filter(|cmd| !cmd.packages_used.is_empty())
            .collect();

        let managers_used = self.analyze_package_managers(&package_commands);
        let package_trends = self.identify_package_trends(&package_commands);
        let version_conflicts = self.detect_version_conflicts(&package_commands);
        let recommendations =
            self.generate_recommendations(&managers_used, &package_trends, &version_conflicts);

        PackageAnalysis {
            total_package_operations: package_commands.len(),
            managers_used,
            package_trends,
            version_conflicts,
            recommendations,
        }
    }

    fn analyze_package_managers(&self, commands: &[&Command]) -> Vec<ManagerStats> {
        let mut manager_data: HashMap<
            String,
            (usize, usize, usize, HashMap<String, PackageStats>),
        > = HashMap::new();
        // (installs, removes, updates, package_stats)

        for cmd in commands {
            for package in &cmd.packages_used {
                let entry = manager_data.entry(package.manager.clone()).or_insert((
                    0,
                    0,
                    0,
                    HashMap::new(),
                ));

                match package.action.as_str() {
                    "install" => entry.0 += 1,
                    "remove" | "uninstall" => entry.1 += 1,
                    "update" | "upgrade" => entry.2 += 1,
                    _ => entry.0 += 1, // Default to install
                }

                // Track individual package stats
                let pkg_stats = entry.3.entry(package.name.clone()).or_insert(PackageStats {
                    name: package.name.clone(),
                    install_count: 0,
                    remove_count: 0,
                    first_installed: None,
                    last_used: cmd.timestamp,
                    versions_seen: Vec::new(),
                });

                match package.action.as_str() {
                    "install" => {
                        pkg_stats.install_count += 1;
                        if pkg_stats.first_installed.is_none() {
                            pkg_stats.first_installed = Some(cmd.timestamp);
                        }
                    }
                    "remove" | "uninstall" => pkg_stats.remove_count += 1,
                    _ => {}
                }

                pkg_stats.last_used = pkg_stats.last_used.max(cmd.timestamp);

                if let Some(version) = &package.version {
                    if !pkg_stats.versions_seen.contains(version) {
                        pkg_stats.versions_seen.push(version.clone());
                    }
                }
            }
        }

        let mut managers: Vec<_> = manager_data
            .into_iter()
            .map(|(manager, (installs, removes, updates, packages))| {
                let mut top_packages: Vec<_> = packages.into_values().collect();
                top_packages.sort_by(|a, b| b.install_count.cmp(&a.install_count));
                top_packages.truncate(10);

                ManagerStats {
                    manager,
                    total_operations: installs + removes + updates,
                    installs,
                    removes,
                    updates,
                    top_packages,
                }
            })
            .collect();

        managers.sort_by(|a, b| b.total_operations.cmp(&a.total_operations));
        managers
    }

    fn identify_package_trends(&self, commands: &[&Command]) -> Vec<PackageTrend> {
        let mut trends = Vec::new();
        type PackageTimeline = HashMap<(String, String), Vec<(DateTime<Utc>, String)>>;
        let mut package_timeline: PackageTimeline = HashMap::new();

        // Build timeline for each package
        for cmd in commands {
            for package in &cmd.packages_used {
                let key = (package.manager.clone(), package.name.clone());
                package_timeline
                    .entry(key)
                    .or_default()
                    .push((cmd.timestamp, package.action.clone()));
            }
        }

        // Analyze trends
        for ((manager, package_name), timeline) in package_timeline {
            let installs = timeline
                .iter()
                .filter(|(_, action)| action == "install")
                .count();
            let removes = timeline
                .iter()
                .filter(|(_, action)| action == "remove" || action == "uninstall")
                .count();

            // Frequent installs
            if installs >= 3 {
                trends.push(PackageTrend {
                    package: package_name.clone(),
                    manager: manager.clone(),
                    trend_type: TrendType::FrequentInstalls,
                    frequency: installs,
                    time_span_days: self.calculate_time_span(&timeline),
                });
            }

            // Repeated installs (same package installed multiple times)
            if installs > removes + 1 {
                trends.push(PackageTrend {
                    package: package_name.clone(),
                    manager: manager.clone(),
                    trend_type: TrendType::RepeatedInstalls,
                    frequency: installs - removes,
                    time_span_days: self.calculate_time_span(&timeline),
                });
            }

            // Quick removal (installed then quickly removed)
            if let Some(quick_removal) = self.detect_quick_removal(&timeline) {
                trends.push(quick_removal);
            }
        }

        trends.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        trends.truncate(20);
        trends
    }

    fn calculate_time_span(&self, timeline: &[(DateTime<Utc>, String)]) -> i64 {
        if timeline.len() < 2 {
            return 0;
        }

        let first = timeline.iter().map(|(time, _)| time).min().unwrap();
        let last = timeline.iter().map(|(time, _)| time).max().unwrap();

        (*last - *first).num_days()
    }

    fn detect_quick_removal(&self, timeline: &[(DateTime<Utc>, String)]) -> Option<PackageTrend> {
        // Look for install followed by remove within 24 hours
        for i in 0..timeline.len().saturating_sub(1) {
            if timeline[i].1 == "install" {
                for j in i + 1..timeline.len() {
                    if timeline[j].1 == "remove" || timeline[j].1 == "uninstall" {
                        let time_diff = timeline[j].0 - timeline[i].0;
                        if time_diff.num_hours() <= 24 {
                            // Quick removal detected
                            return Some(PackageTrend {
                                package: "".to_string(), // Would need package name from context
                                manager: "".to_string(), // Would need manager from context
                                trend_type: TrendType::QuickRemoval,
                                frequency: 1,
                                time_span_days: 0,
                            });
                        }
                        break;
                    }
                }
            }
        }
        None
    }

    fn detect_version_conflicts(&self, commands: &[&Command]) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();
        let mut package_versions: HashMap<(String, String), Vec<String>> = HashMap::new();

        // Collect all versions seen for each package
        for cmd in commands {
            for package in &cmd.packages_used {
                if let Some(version) = &package.version {
                    let key = (package.manager.clone(), package.name.clone());
                    let versions = package_versions.entry(key).or_default();
                    if !versions.contains(version) {
                        versions.push(version.clone());
                    }
                }
            }
        }

        // Identify conflicts
        for ((manager, package), versions) in package_versions {
            if versions.len() > 1 {
                let conflict_type = if self.has_version_downgrade(&versions) {
                    ConflictType::DowngradeDetected
                } else {
                    ConflictType::MultipleVersions
                };

                let recommendation = match conflict_type {
                    ConflictType::DowngradeDetected => {
                        "Consider if downgrade was intentional. Check for compatibility issues."
                            .to_string()
                    }
                    ConflictType::MultipleVersions => {
                        "Multiple versions detected. Consider standardizing on one version."
                            .to_string()
                    }
                    ConflictType::InconsistentVersioning => {
                        "Inconsistent versioning scheme detected.".to_string()
                    }
                };

                conflicts.push(VersionConflict {
                    package,
                    manager,
                    versions,
                    conflict_type,
                    recommendation,
                });
            }
        }

        conflicts
    }

    fn has_version_downgrade(&self, versions: &[String]) -> bool {
        // Simple heuristic: if we see a lower version after a higher one
        // This is a simplified check and would need more sophisticated version parsing
        versions.len() > 1
            && versions
                .iter()
                .any(|v| v.contains("0.") || v.starts_with("1."))
    }

    fn generate_recommendations(
        &self,
        managers: &[ManagerStats],
        trends: &[PackageTrend],
        conflicts: &[VersionConflict],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Manager-specific recommendations
        for manager in managers {
            if manager.removes > manager.installs / 2 {
                recommendations.push(format!(
                    "📦 High removal rate for {} packages - consider testing before installing",
                    manager.manager
                ));
            }

            if manager.manager == "npm" && manager.installs > 20 {
                recommendations.push(
                    "📦 Consider using npm ci for faster, reliable installs in CI/CD".to_string(),
                );
            }

            if manager.manager == "pip" && manager.installs > 15 {
                recommendations.push(
                    "🐍 Consider using virtual environments to isolate Python dependencies"
                        .to_string(),
                );
            }
        }

        // Trend-based recommendations
        for trend in trends.iter().take(5) {
            match trend.trend_type {
                TrendType::RepeatedInstalls => {
                    recommendations.push(format!(
                        "🔄 Package '{}' installed {} times - check if this is intentional",
                        trend.package, trend.frequency
                    ));
                }
                TrendType::FrequentInstalls => {
                    recommendations.push(format!(
                        "📈 Frequent installs of '{}' - consider adding to requirements file",
                        trend.package
                    ));
                }
                _ => {}
            }
        }

        // Conflict-based recommendations
        if !conflicts.is_empty() {
            recommendations.push(format!(
                "⚠️ {} version conflicts detected - review package versions for consistency",
                conflicts.len()
            ));
        }

        // General recommendations
        if managers.len() > 3 {
            recommendations.push(
                "🔧 Multiple package managers in use - consider standardizing where possible"
                    .to_string(),
            );
        }

        recommendations.truncate(8);
        recommendations
    }

    pub fn calculate_package_health_score(&self, analysis: &PackageAnalysis) -> f32 {
        if analysis.total_package_operations == 0 {
            return 1.0;
        }

        let mut score = 1.0;

        // Penalty for version conflicts
        let conflict_penalty = (analysis.version_conflicts.len() as f32 * 0.1).min(0.3);
        score -= conflict_penalty;

        // Penalty for excessive repeated installs
        let repeated_installs = analysis
            .package_trends
            .iter()
            .filter(|t| matches!(t.trend_type, TrendType::RepeatedInstalls))
            .count() as f32;
        let repeat_penalty = (repeated_installs * 0.05).min(0.2);
        score -= repeat_penalty;

        // Bonus for consistent package management
        let total_ops = analysis
            .managers_used
            .iter()
            .map(|m| m.total_operations)
            .sum::<usize>() as f32;
        let primary_manager_ops = analysis
            .managers_used
            .first()
            .map(|m| m.total_operations as f32)
            .unwrap_or(0.0);

        if total_ops > 0.0 {
            let consistency_ratio = primary_manager_ops / total_ops;
            if consistency_ratio > 0.7 {
                score += 0.1; // Bonus for consistent tool usage
            }
        }

        score.clamp(0.0, 1.0)
    }
}
