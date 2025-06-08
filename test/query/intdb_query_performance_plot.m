%% IntDB vs InfluxDB Query Performance Analysis
% Simplified MATLAB plotting script
% Author: IntDB Research Team
% Date: June 2025

close all; clear; clc;

%% Configuration
% Set chart style
set(groot, 'defaultAxesColorOrder', [0 0.4470 0.7410; 0.8500 0.3250 0.0980; 0.9290 0.6940 0.1250]);
set(groot, 'defaultLineLineWidth', 2);
set(groot, 'defaultAxesFontSize', 12);
set(groot, 'defaultFigureColor', 'white');
set(groot, 'defaultAxesColor', 'white');

%% Experimental Data
fprintf('IntDB vs InfluxDB Query Performance Analysis\n');
fprintf('===========================================\n\n');

% Experiment configurations
experiments = {'Exp1 (1K)', 'Exp2 (5K)', 'Exp3 (10K)'};
data_scales = [1000, 5000, 10000];

% Query performance data (milliseconds)
% Path Pattern Matching
pattern_intdb = [34.43, 34.11, 45.68];
pattern_influxdb = [141.88, 209.21, 371.02];
pattern_intdb_p95 = [36.19, 39.34, 60.53];
pattern_influxdb_p95 = [149.08, 236.07, 495.43];

% Path Aggregation
aggregation_intdb = [7.43, 6.82, 14.81];
aggregation_influxdb = [32.02, 48.87, 84.13];
aggregation_intdb_p95 = [10.82, 7.97, 19.27];
aggregation_influxdb_p95 = [36.89, 57.07, 91.21];

% Performance improvement percentages
pattern_improvement = [75.7, 83.7, 87.7];
aggregation_improvement = [76.8, 86.0, 82.4];

%% Query Performance Analysis Plot
fprintf('Creating query performance analysis chart...\n');

figure('Name', 'Query Performance Analysis', 'Position', [150, 150, 1200, 400], 'Color', 'white');

x = 1:length(experiments);
width = 0.4;

% Subplot 1: Path Pattern Matching - Average Response Time
subplot(1, 3, 1);
b1 = bar(x - width/2, pattern_intdb, width, 'FaceColor', [0 0.4470 0.7410], 'DisplayName', 'IntDB');
hold on;
b2 = bar(x + width/2, pattern_influxdb, width, 'FaceColor', [0.8500 0.3250 0.0980], 'DisplayName', 'InfluxDB');

% Add value labels on top of bars
for i = 1:length(pattern_intdb)
    text(i - width/2, pattern_intdb(i) + max(pattern_influxdb) * 0.15, ...
         sprintf('%.1f', pattern_intdb(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
    text(i + width/2, pattern_influxdb(i) + max(pattern_influxdb) * 0.15, ...
         sprintf('%.1f', pattern_influxdb(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
end

ylabel('Avg Pattern Match Time (ms)');
set(gca, 'XTick', x);
set(gca, 'XTickLabel', experiments);
xlim([0.5, length(experiments)+0.5]);
ylim([0, max(pattern_influxdb) * 1.25]);
legend('Location', 'northwest');
grid on;

% Subplot 2: Path Pattern Matching - P95 Response Time
subplot(1, 3, 2);
bar(x - width/2, pattern_intdb_p95, width, 'FaceColor', [0 0.4470 0.7410], 'DisplayName', 'IntDB');
hold on;
bar(x + width/2, pattern_influxdb_p95, width, 'FaceColor', [0.8500 0.3250 0.0980], 'DisplayName', 'InfluxDB');

% Add value labels on top of bars
for i = 1:length(pattern_intdb_p95)
    text(i - width/2, pattern_intdb_p95(i) + max(pattern_influxdb_p95) * 0.15, ...
         sprintf('%.1f', pattern_intdb_p95(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
    text(i + width/2, pattern_influxdb_p95(i) + max(pattern_influxdb_p95) * 0.15, ...
         sprintf('%.1f', pattern_influxdb_p95(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
end

ylabel('P95 Pattern Match Time (ms)');
set(gca, 'XTick', x);
set(gca, 'XTickLabel', experiments);
xlim([0.5, length(experiments)+0.5]);
ylim([0, max(pattern_influxdb_p95) * 1.25]);
legend('Location', 'northwest');
grid on;

% Subplot 3: Path Aggregation - Average Response Time
subplot(1, 3, 3);
bar(x - width/2, aggregation_intdb, width, 'FaceColor', [0 0.4470 0.7410], 'DisplayName', 'IntDB');
hold on;
bar(x + width/2, aggregation_influxdb, width, 'FaceColor', [0.8500 0.3250 0.0980], 'DisplayName', 'InfluxDB');

% Add value labels on top of bars
for i = 1:length(aggregation_intdb)
    text(i - width/2, aggregation_intdb(i) + max(aggregation_influxdb) * 0.15, ...
         sprintf('%.1f', aggregation_intdb(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
    text(i + width/2, aggregation_influxdb(i) + max(aggregation_influxdb) * 0.15, ...
         sprintf('%.1f', aggregation_influxdb(i)), ...
         'HorizontalAlignment', 'center', 'FontSize', 12, 'Rotation', 45);
end

ylabel('Avg Aggregation Time (ms)');
set(gca, 'XTick', x);
set(gca, 'XTickLabel', experiments);
xlim([0.5, length(experiments)+0.5]);
ylim([0, max(aggregation_influxdb) * 1.25]);
legend('Location', 'northwest');
grid on;

