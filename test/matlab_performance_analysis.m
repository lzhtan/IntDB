%% IntDB vs InfluxDB 性能测试结果分析
% MATLAB绘图脚本
% 作者: IntDB项目组
% 时间: 2025年

close all; clear; clc;

%% 配置
% 设置数据路径 - 请根据实际路径修改
data_path = 'test/performance_results_20250606_220608/';

% 设置图表样式
set(groot, 'defaultAxesColorOrder', [0 0.4470 0.7410; 0.8500 0.3250 0.0980; 0.9290 0.6940 0.1250]);
set(groot, 'defaultLineLineWidth', 2);
set(groot, 'defaultAxesFontSize', 12);

%% 1. 并发扩展性分析
fprintf('正在分析并发扩展性...\n');

% 读取并发测试数据
concurrency_file = fullfile(data_path, 'concurrency_scaling.csv');
if exist(concurrency_file, 'file')
    concurrency_data = readtable(concurrency_file);
    
    % 分离IntDB和InfluxDB数据
    intdb_conc = concurrency_data(strcmp(concurrency_data.database, 'IntDB'), :);
    influxdb_conc = concurrency_data(strcmp(concurrency_data.database, 'InfluxDB'), :);
    
    % 创建并发扩展性图表
    figure('Name', '并发扩展性分析', 'Position', [100, 100, 1400, 900]);
    
    % 子图1: 响应时间 vs 并发数
    subplot(2, 3, 1);
    plot(intdb_conc.concurrency, intdb_conc.response_time, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.response_time, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('响应时间 (ms)');
    title('响应时间扩展性');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % 子图2: QPS vs 并发数
    subplot(2, 3, 2);
    plot(intdb_conc.concurrency, intdb_conc.qps, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.qps, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('QPS (req/sec)');
    title('吞吐量扩展性');
    legend('Location', 'northeast');
    grid on;
    set(gca, 'XScale', 'log');
    
    % 子图3: 可用性 vs 并发数
    subplot(2, 3, 3);
    plot(intdb_conc.concurrency, intdb_conc.availability, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.availability, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('可用性 (%)');
    title('可用性稳定性');
    legend('Location', 'southwest');
    grid on;
    set(gca, 'XScale', 'log');
    ylim([0, 105]);
    
    % 子图4: 失败事务数 vs 并发数
    subplot(2, 3, 4);
    semilogy(intdb_conc.concurrency, max(intdb_conc.failed_transactions, 0.1), 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    semilogy(influxdb_conc.concurrency, max(influxdb_conc.failed_transactions, 0.1), 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('失败事务数 (对数尺度)');
    title('错误率对比');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    
    % 子图5: 最大延迟 vs 并发数
    subplot(2, 3, 5);
    plot(intdb_conc.concurrency, intdb_conc.max_latency, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.max_latency, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('最大延迟 (ms)');
    title('延迟峰值对比');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % 子图6: 并发效率（QPS/并发数）
    subplot(2, 3, 6);
    intdb_efficiency = intdb_conc.qps ./ intdb_conc.concurrency;
    influxdb_efficiency = influxdb_conc.qps ./ influxdb_conc.concurrency;
    plot(intdb_conc.concurrency, intdb_efficiency, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_efficiency, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('并发数');
    ylabel('效率 (QPS/并发数)');
    title('并发效率对比');
    legend('Location', 'northeast');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % 保存图表
    saveas(gcf, fullfile(data_path, 'matlab_concurrency_analysis.png'));
    fprintf('✅ 并发扩展性分析完成，图表已保存\n');
else
    fprintf('❌ 未找到并发测试数据文件\n');
end

%% 2. 持续时间扩展性分析
fprintf('正在分析持续时间扩展性...\n');

duration_file = fullfile(data_path, 'duration_scaling.csv');
if exist(duration_file, 'file')
    duration_data = readtable(duration_file);
    
    % 分离数据
    intdb_dur = duration_data(strcmp(duration_data.database, 'IntDB'), :);
    influxdb_dur = duration_data(strcmp(duration_data.database, 'InfluxDB'), :);
    
    % 创建持续时间分析图表
    figure('Name', '持续时间扩展性分析', 'Position', [150, 150, 1200, 800]);
    
    % 子图1: QPS vs 测试时长
    subplot(2, 2, 1);
    plot(intdb_dur.duration, intdb_dur.qps, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.qps, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('测试时长 (秒)');
    ylabel('平均QPS');
    title('长期QPS稳定性');
    legend('Location', 'best');
    grid on;
    
    % 子图2: 响应时间 vs 测试时长
    subplot(2, 2, 2);
    plot(intdb_dur.duration, intdb_dur.response_time, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.response_time, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('测试时长 (秒)');
    ylabel('平均响应时间 (ms)');
    title('长期响应时间稳定性');
    legend('Location', 'best');
    grid on;
    
    % 子图3: 总事务数 vs 测试时长
    subplot(2, 2, 3);
    plot(intdb_dur.duration, intdb_dur.transactions, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.transactions, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('测试时长 (秒)');
    ylabel('总事务数');
    title('总处理能力对比');
    legend('Location', 'northwest');
    grid on;
    
    % 子图4: 错误率 vs 测试时长
    subplot(2, 2, 4);
    intdb_error_rate = (intdb_dur.failed_transactions ./ intdb_dur.transactions) * 100;
    influxdb_error_rate = (influxdb_dur.failed_transactions ./ influxdb_dur.transactions) * 100;
    plot(intdb_dur.duration, intdb_error_rate, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_error_rate, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('测试时长 (秒)');
    ylabel('错误率 (%)');
    title('长期错误率趋势');
    legend('Location', 'best');
    grid on;
    
    % 保存图表
    saveas(gcf, fullfile(data_path, 'matlab_duration_analysis.png'));
    fprintf('✅ 持续时间分析完成，图表已保存\n');
else
    fprintf('❌ 未找到持续时间测试数据文件\n');
end

%% 3. 功能端点性能对比
fprintf('正在分析功能端点性能...\n');

endpoint_file = fullfile(data_path, 'functional_endpoints.csv');
if exist(endpoint_file, 'file')
    endpoint_data = readtable(endpoint_file);
    
    % 获取唯一端点
    unique_endpoints = unique(endpoint_data.endpoint);
    
    % 创建端点对比图表
    figure('Name', '功能端点性能对比', 'Position', [200, 200, 1200, 800]);
    
    % 准备数据
    intdb_resp_times = [];
    influxdb_resp_times = [];
    intdb_qps_vals = [];
    influxdb_qps_vals = [];
    intdb_avail = [];
    influxdb_avail = [];
    endpoint_labels = {};
    
    for i = 1:length(unique_endpoints)
        ep = unique_endpoints{i};
        
        % IntDB数据
        intdb_row = endpoint_data(strcmp(endpoint_data.endpoint, ep) & strcmp(endpoint_data.database, 'IntDB'), :);
        if ~isempty(intdb_row)
            intdb_resp_times(end+1) = intdb_row.response_time(1);
            intdb_qps_vals(end+1) = intdb_row.qps(1);
            intdb_avail(end+1) = intdb_row.availability(1);
            endpoint_labels{end+1} = ['IntDB' ep];
        end
        
        % InfluxDB数据
        influxdb_row = endpoint_data(strcmp(endpoint_data.endpoint, ep) & strcmp(endpoint_data.database, 'InfluxDB'), :);
        if ~isempty(influxdb_row)
            influxdb_resp_times(end+1) = influxdb_row.response_time(1);
            influxdb_qps_vals(end+1) = influxdb_row.qps(1);
            influxdb_avail(end+1) = influxdb_row.availability(1);
            endpoint_labels{end+1} = ['InfluxDB' ep];
        end
    end
    
    % 子图1: 响应时间对比
    subplot(2, 2, 1);
    x_pos = 1:length(endpoint_labels);
    all_resp_times = [intdb_resp_times, influxdb_resp_times];
    bar_colors = repmat([0 0.4470 0.7410], length(intdb_resp_times), 1);
    bar_colors = [bar_colors; repmat([0.8500 0.3250 0.0980], length(influxdb_resp_times), 1)];
    
    b1 = bar(x_pos, all_resp_times, 'FaceColor', 'flat');
    b1.CData = bar_colors;
    xlabel('端点');
    ylabel('响应时间 (ms)');
    title('端点响应时间对比');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    grid on;
    
    % 子图2: QPS对比
    subplot(2, 2, 2);
    all_qps = [intdb_qps_vals, influxdb_qps_vals];
    b2 = bar(x_pos, all_qps, 'FaceColor', 'flat');
    b2.CData = bar_colors;
    xlabel('端点');
    ylabel('QPS');
    title('端点吞吐量对比');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    grid on;
    
    % 子图3: 可用性对比
    subplot(2, 2, 3);
    all_avail = [intdb_avail, influxdb_avail];
    b3 = bar(x_pos, all_avail, 'FaceColor', 'flat');
    b3.CData = bar_colors;
    xlabel('端点');
    ylabel('可用性 (%)');
    title('端点可用性对比');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    ylim([99, 100.1]);
    grid on;
    
    % 子图4: 综合性能雷达图 (简化版)
    subplot(2, 2, 4);
    if ~isempty(intdb_resp_times) && ~isempty(influxdb_resp_times)
        % 计算平均值
        intdb_avg_resp = mean(intdb_resp_times);
        influxdb_avg_resp = mean(influxdb_resp_times);
        intdb_avg_qps = mean(intdb_qps_vals);
        influxdb_avg_qps = mean(influxdb_qps_vals);
        intdb_avg_avail = mean(intdb_avail);
        influxdb_avg_avail = mean(influxdb_avail);
        
        % 标准化数据 (0-1范围)
        max_resp = max(intdb_avg_resp, influxdb_avg_resp);
        max_qps = max(intdb_avg_qps, influxdb_avg_qps);
        
        intdb_norm = [1 - intdb_avg_resp/max_resp, intdb_avg_qps/max_qps, intdb_avg_avail/100];
        influxdb_norm = [1 - influxdb_avg_resp/max_resp, influxdb_avg_qps/max_qps, influxdb_avg_avail/100];
        
        categories = {'响应时间', 'QPS', '可用性'};
        angles = linspace(0, 2*pi, length(categories)+1);
        
        % 绘制雷达图
        polarplot(angles, [intdb_norm, intdb_norm(1)], 'bo-', 'LineWidth', 2, 'MarkerSize', 8);
        hold on;
        polarplot(angles, [influxdb_norm, influxdb_norm(1)], 'rs-', 'LineWidth', 2, 'MarkerSize', 8);
        
        % 设置角度标签
        thetaticks(rad2deg(angles(1:end-1)));
        thetaticklabels(categories);
        title('综合性能雷达图');
        legend('IntDB', 'InfluxDB', 'Location', 'best');
    end
    
    % 保存图表
    saveas(gcf, fullfile(data_path, 'matlab_endpoint_analysis.png'));
    fprintf('✅ 功能端点分析完成，图表已保存\n');
else
    fprintf('❌ 未找到功能端点测试数据文件\n');
end

%% 4. 突发负载测试分析
fprintf('正在分析突发负载性能...\n');

burst_file = fullfile(data_path, 'burst_load.csv');
if exist(burst_file, 'file')
    burst_data = readtable(burst_file);
    
    % 分离数据
    intdb_burst = burst_data(strcmp(burst_data.database, 'IntDB'), :);
    influxdb_burst = burst_data(strcmp(burst_data.database, 'InfluxDB'), :);
    
    % 创建突发负载分析图表
    figure('Name', '突发负载测试分析', 'Position', [250, 250, 1000, 600]);
    
    phases = {'low', 'burst', 'recovery'};
    phase_labels = {'低负载', '突发负载', '恢复阶段'};
    
    % 子图1: 响应时间对比
    subplot(2, 2, 1);
    intdb_resp = [];
    influxdb_resp = [];
    for i = 1:length(phases)
        intdb_row = intdb_burst(strcmp(intdb_burst.phase, phases{i}), :);
        influxdb_row = influxdb_burst(strcmp(influxdb_burst.phase, phases{i}), :);
        
        if ~isempty(intdb_row)
            intdb_resp(i) = intdb_row.response_time(1);
        else
            intdb_resp(i) = 0;
        end
        
        if ~isempty(influxdb_row)
            influxdb_resp(i) = influxdb_row.response_time(1);
        else
            influxdb_resp(i) = 0;
        end
    end
    
    x = 1:length(phases);
    width = 0.35;
    bar(x - width/2, intdb_resp, width, 'DisplayName', 'IntDB');
    hold on;
    bar(x + width/2, influxdb_resp, width, 'DisplayName', 'InfluxDB');
    xlabel('负载阶段');
    ylabel('响应时间 (ms)');
    title('突发负载响应时间');
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % 子图2: QPS对比
    subplot(2, 2, 2);
    intdb_qps_burst = [];
    influxdb_qps_burst = [];
    for i = 1:length(phases)
        intdb_row = intdb_burst(strcmp(intdb_burst.phase, phases{i}), :);
        influxdb_row = influxdb_burst(strcmp(influxdb_burst.phase, phases{i}), :);
        
        if ~isempty(intdb_row)
            intdb_qps_burst(i) = intdb_row.qps(1);
        else
            intdb_qps_burst(i) = 0;
        end
        
        if ~isempty(influxdb_row)
            influxdb_qps_burst(i) = influxdb_row.qps(1);
        else
            influxdb_qps_burst(i) = 0;
        end
    end
    
    bar(x - width/2, intdb_qps_burst, width, 'DisplayName', 'IntDB');
    hold on;
    bar(x + width/2, influxdb_qps_burst, width, 'DisplayName', 'InfluxDB');
    xlabel('负载阶段');
    ylabel('QPS');
    title('突发负载吞吐量');
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % 子图3: 时间序列图
    subplot(2, 1, 2);
    plot(1:3, intdb_resp, 'bo-', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'IntDB响应时间');
    hold on;
    plot(1:3, influxdb_resp, 'rs-', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'InfluxDB响应时间');
    
    yyaxis right;
    plot(1:3, intdb_qps_burst, 'b^--', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'IntDB QPS');
    plot(1:3, influxdb_qps_burst, 'rd--', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'InfluxDB QPS');
    ylabel('QPS');
    
    yyaxis left;
    ylabel('响应时间 (ms)');
    xlabel('负载阶段');
    title('突发负载时间序列分析');
    set(gca, 'XTick', 1:3);
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % 保存图表
    saveas(gcf, fullfile(data_path, 'matlab_burst_analysis.png'));
    fprintf('✅ 突发负载分析完成，图表已保存\n');
else
    fprintf('❌ 未找到突发负载测试数据文件\n');
end

%% 5. 生成性能摘要统计
fprintf('正在生成性能摘要统计...\n');

% 创建摘要报告
summary_file = fullfile(data_path, 'matlab_performance_summary.txt');
fid = fopen(summary_file, 'w');

fprintf(fid, 'IntDB vs InfluxDB 性能测试摘要报告\n');
fprintf(fid, '=====================================\n\n');
fprintf(fid, '报告生成时间: %s\n\n', datestr(now));

% 并发测试摘要
if exist('intdb_conc', 'var') && exist('influxdb_conc', 'var')
    fprintf(fid, '1. 并发扩展性测试摘要\n');
    fprintf(fid, '----------------------\n');
    fprintf(fid, 'IntDB平均响应时间: %.2f ms\n', mean(intdb_conc.response_time));
    fprintf(fid, 'InfluxDB平均响应时间: %.2f ms\n', mean(influxdb_conc.response_time));
    fprintf(fid, 'IntDB平均QPS: %.1f\n', mean(intdb_conc.qps));
    fprintf(fid, 'InfluxDB平均QPS: %.1f\n', mean(influxdb_conc.qps));
    fprintf(fid, 'IntDB最佳QPS: %.1f (并发数: %d)\n', max(intdb_conc.qps), intdb_conc.concurrency(intdb_conc.qps == max(intdb_conc.qps)));
    fprintf(fid, 'InfluxDB最佳QPS: %.1f (并发数: %d)\n\n', max(influxdb_conc.qps), influxdb_conc.concurrency(influxdb_conc.qps == max(influxdb_conc.qps)));
end

% 持续时间测试摘要
if exist('intdb_dur', 'var') && exist('influxdb_dur', 'var')
    fprintf(fid, '2. 持续时间测试摘要\n');
    fprintf(fid, '--------------------\n');
    fprintf(fid, 'IntDB长期平均QPS: %.1f\n', mean(intdb_dur.qps));
    fprintf(fid, 'InfluxDB长期平均QPS: %.1f\n', mean(influxdb_dur.qps));
    fprintf(fid, 'IntDB长期平均响应时间: %.2f ms\n', mean(intdb_dur.response_time));
    fprintf(fid, 'InfluxDB长期平均响应时间: %.2f ms\n\n', mean(influxdb_dur.response_time));
end

fprintf(fid, '3. 生成的图表文件\n');
fprintf(fid, '------------------\n');
fprintf(fid, '- matlab_concurrency_analysis.png\n');
fprintf(fid, '- matlab_duration_analysis.png\n');
fprintf(fid, '- matlab_endpoint_analysis.png\n');
fprintf(fid, '- matlab_burst_analysis.png\n\n');

fprintf(fid, '注意：所有图表已保存在测试结果目录中\n');

fclose(fid);
fprintf('✅ 性能摘要报告已保存: %s\n', summary_file);

%% 完成
fprintf('\n🎉 MATLAB性能分析完成！\n');
fprintf('📊 生成的文件:\n');
fprintf('   - matlab_concurrency_analysis.png\n');
fprintf('   - matlab_duration_analysis.png\n');
fprintf('   - matlab_endpoint_analysis.png\n');
fprintf('   - matlab_burst_analysis.png\n');
fprintf('   - matlab_performance_summary.txt\n');
fprintf('\n请检查 %s 目录中的图表文件\n', data_path); 