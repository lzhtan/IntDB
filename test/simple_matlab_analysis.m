%% 简化版 IntDB vs InfluxDB 性能分析
% 快速生成基本对比图表

clear; close all; clc;

% 数据路径
data_path = 'test/performance_results_20250606_220608/';

%% 1. 基本并发性能对比
fprintf('正在生成基本性能对比图...\n');

% 读取数据
concurrency_data = readtable(fullfile(data_path, 'concurrency_scaling.csv'));

% 分离数据
intdb_data = concurrency_data(strcmp(concurrency_data.database, 'IntDB'), :);
influxdb_data = concurrency_data(strcmp(concurrency_data.database, 'InfluxDB'), :);

% 创建对比图
figure('Position', [100, 100, 1200, 800]);

% 响应时间对比
subplot(2, 2, 1);
loglog(intdb_data.concurrency, intdb_data.response_time, 'bo-', 'LineWidth', 2, 'MarkerSize', 8);
hold on;
loglog(influxdb_data.concurrency, influxdb_data.response_time, 'rs-', 'LineWidth', 2, 'MarkerSize', 8);
xlabel('并发数');
ylabel('响应时间 (ms)');
title('响应时间对比 (对数尺度)');
legend('IntDB', 'InfluxDB', 'Location', 'northwest');
grid on;

% QPS对比
subplot(2, 2, 2);
semilogx(intdb_data.concurrency, intdb_data.qps, 'bo-', 'LineWidth', 2, 'MarkerSize', 8);
hold on;
semilogx(influxdb_data.concurrency, influxdb_data.qps, 'rs-', 'LineWidth', 2, 'MarkerSize', 8);
xlabel('并发数');
ylabel('QPS');
title('吞吐量对比');
legend('IntDB', 'InfluxDB', 'Location', 'best');
grid on;

% 可用性对比
subplot(2, 2, 3);
semilogx(intdb_data.concurrency, intdb_data.availability, 'bo-', 'LineWidth', 2, 'MarkerSize', 8);
hold on;
semilogx(influxdb_data.concurrency, influxdb_data.availability, 'rs-', 'LineWidth', 2, 'MarkerSize', 8);
xlabel('并发数');
ylabel('可用性 (%)');
title('可用性对比');
legend('IntDB', 'InfluxDB', 'Location', 'southwest');
grid on;
ylim([50, 105]);

% 性能摘要柱状图
subplot(2, 2, 4);
categories = {'平均响应时间', '平均QPS', '最大QPS'};
intdb_values = [mean(intdb_data.response_time), mean(intdb_data.qps), max(intdb_data.qps)];
influxdb_values = [mean(influxdb_data.response_time), mean(influxdb_data.qps), max(influxdb_data.qps)];

% 标准化数据用于显示
norm_intdb = [intdb_values(1)/max(intdb_values(1), influxdb_values(1)), ...
              intdb_values(2)/max(intdb_values(2), influxdb_values(2)), ...
              intdb_values(3)/max(intdb_values(3), influxdb_values(3))];
norm_influxdb = [influxdb_values(1)/max(intdb_values(1), influxdb_values(1)), ...
                 influxdb_values(2)/max(intdb_values(2), influxdb_values(2)), ...
                 influxdb_values(3)/max(intdb_values(3), influxdb_values(3))];

x = 1:3;
width = 0.35;
bar(x - width/2, norm_intdb, width, 'DisplayName', 'IntDB');
hold on;
bar(x + width/2, norm_influxdb, width, 'DisplayName', 'InfluxDB');
xlabel('性能指标');
ylabel('标准化值');
title('综合性能对比');
set(gca, 'XTickLabel', categories);
xtickangle(45);
legend('Location', 'best');
grid on;

% 保存图表
saveas(gcf, fullfile(data_path, 'simple_performance_comparison.png'));

%% 2. 性能数据摘要表格
fprintf('\n=== 性能测试结果摘要 ===\n');
fprintf('IntDB平均响应时间: %.2f ms\n', mean(intdb_data.response_time));
fprintf('InfluxDB平均响应时间: %.2f ms\n', mean(influxdb_data.response_time));
fprintf('响应时间差异: %.1f%%\n', (mean(influxdb_data.response_time) - mean(intdb_data.response_time)) / mean(influxdb_data.response_time) * 100);

fprintf('\nIntDB平均QPS: %.1f\n', mean(intdb_data.qps));
fprintf('InfluxDB平均QPS: %.1f\n', mean(influxdb_data.qps));
fprintf('QPS差异: %.1f%%\n', (mean(intdb_data.qps) - mean(influxdb_data.qps)) / mean(influxdb_data.qps) * 100);

fprintf('\nIntDB最大QPS: %.1f (并发数: %d)\n', max(intdb_data.qps), intdb_data.concurrency(intdb_data.qps == max(intdb_data.qps)));
fprintf('InfluxDB最大QPS: %.1f (并发数: %d)\n', max(influxdb_data.qps), influxdb_data.concurrency(influxdb_data.qps == max(influxdb_data.qps)));

fprintf('\nIntDB平均可用性: %.2f%%\n', mean(intdb_data.availability));
fprintf('InfluxDB平均可用性: %.2f%%\n', mean(influxdb_data.availability));

fprintf('\n✅ 图表已保存: %s\n', fullfile(data_path, 'simple_performance_comparison.png')); 