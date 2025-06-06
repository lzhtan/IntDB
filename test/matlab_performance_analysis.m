%% IntDB vs InfluxDB Performance Test Results Analysis
% MATLAB plotting script
% Author: IntDB Project Team
% Date: 2025

close all; clear; clc;

%% Configuration
% Set data path - please modify according to actual path
data_path = 'performance_results_20250606_220608/';

% Set chart style
set(groot, 'defaultAxesColorOrder', [0 0.4470 0.7410; 0.8500 0.3250 0.0980; 0.9290 0.6940 0.1250]);
set(groot, 'defaultLineLineWidth', 2);
set(groot, 'defaultAxesFontSize', 12);
set(groot, 'defaultFigureColor', 'white');
set(groot, 'defaultAxesColor', 'white');

%% 1. Concurrency Scalability Analysis
fprintf('Analyzing concurrency scalability...\n');

% Read concurrency test data
concurrency_file = fullfile(data_path, 'concurrency_scaling.csv');
if exist(concurrency_file, 'file')
    concurrency_data = readtable(concurrency_file);
    
    % Separate IntDB and InfluxDB data
    intdb_conc = concurrency_data(strcmp(concurrency_data.database, 'IntDB'), :);
    influxdb_conc = concurrency_data(strcmp(concurrency_data.database, 'InfluxDB'), :);
    
    % Create concurrency scalability chart
    figure('Name', 'Concurrency Scalability Analysis', 'Position', [100, 100, 1400, 900], 'Color', 'white');
    
    % Subplot 1: Response Time vs Concurrency
    subplot(2, 3, 1);
    plot(intdb_conc.concurrency, intdb_conc.response_time, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.response_time, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('Response Time (ms)');
    title('Response Time Scalability');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % Subplot 2: QPS vs Concurrency
    subplot(2, 3, 2);
    plot(intdb_conc.concurrency, intdb_conc.qps, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.qps, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('QPS (req/sec)');
    title('Throughput Scalability');
    legend('Location', 'northeast');
    grid on;
    set(gca, 'XScale', 'log');
    
    % Subplot 3: Availability vs Concurrency
    subplot(2, 3, 3);
    plot(intdb_conc.concurrency, intdb_conc.availability, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.availability, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('Availability (%)');
    title('Availability Stability');
    legend('Location', 'southwest');
    grid on;
    set(gca, 'XScale', 'log');
    ylim([0, 105]);
    
    % Subplot 4: Failed Transactions vs Concurrency
    subplot(2, 3, 4);
    semilogy(intdb_conc.concurrency, max(intdb_conc.failed_transactions, 0.1), 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    semilogy(influxdb_conc.concurrency, max(influxdb_conc.failed_transactions, 0.1), 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('Failed Transactions (Log Scale)');
    title('Error Rate Comparison');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    
    % Subplot 5: Max Latency vs Concurrency
    subplot(2, 3, 5);
    plot(intdb_conc.concurrency, intdb_conc.max_latency, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_conc.max_latency, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('Max Latency (ms)');
    title('Peak Latency Comparison');
    legend('Location', 'northwest');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % Subplot 6: Concurrency Efficiency (QPS/Concurrency)
    subplot(2, 3, 6);
    intdb_efficiency = intdb_conc.qps ./ intdb_conc.concurrency;
    influxdb_efficiency = influxdb_conc.qps ./ influxdb_conc.concurrency;
    plot(intdb_conc.concurrency, intdb_efficiency, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_conc.concurrency, influxdb_efficiency, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Concurrency');
    ylabel('Efficiency (QPS/Concurrency)');
    title('Concurrency Efficiency Comparison');
    legend('Location', 'northeast');
    grid on;
    set(gca, 'XScale', 'log');
    set(gca, 'YScale', 'log');
    
    % Save chart
    saveas(gcf, fullfile(data_path, 'matlab_concurrency_analysis.png'));
    fprintf('✅ Concurrency scalability analysis completed, chart saved\n');
else
    fprintf('❌ Concurrency test data file not found\n');
end

%% 2. Duration Scalability Analysis
fprintf('Analyzing duration scalability...\n');

duration_file = fullfile(data_path, 'duration_scaling.csv');
if exist(duration_file, 'file')
    duration_data = readtable(duration_file);
    
    % Separate data
    intdb_dur = duration_data(strcmp(duration_data.database, 'IntDB'), :);
    influxdb_dur = duration_data(strcmp(duration_data.database, 'InfluxDB'), :);
    
    % Create duration analysis chart
    figure('Name', 'Duration Scalability Analysis', 'Position', [150, 150, 1200, 800], 'Color', 'white');
    
    % Subplot 1: QPS vs Test Duration
    subplot(2, 2, 1);
    plot(intdb_dur.duration, intdb_dur.qps, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.qps, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Test Duration (seconds)');
    ylabel('Average QPS');
    title('Long-term QPS Stability');
    legend('Location', 'best');
    grid on;
    
    % Subplot 2: Response Time vs Test Duration
    subplot(2, 2, 2);
    plot(intdb_dur.duration, intdb_dur.response_time, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.response_time, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Test Duration (seconds)');
    ylabel('Average Response Time (ms)');
    title('Long-term Response Time Stability');
    legend('Location', 'best');
    grid on;
    
    % Subplot 3: Total Transactions vs Test Duration
    subplot(2, 2, 3);
    plot(intdb_dur.duration, intdb_dur.transactions, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_dur.transactions, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Test Duration (seconds)');
    ylabel('Total Transactions');
    title('Total Processing Capacity Comparison');
    legend('Location', 'northwest');
    grid on;
    
    % Subplot 4: Error Rate vs Test Duration
    subplot(2, 2, 4);
    intdb_error_rate = (intdb_dur.failed_transactions ./ intdb_dur.transactions) * 100;
    influxdb_error_rate = (influxdb_dur.failed_transactions ./ influxdb_dur.transactions) * 100;
    plot(intdb_dur.duration, intdb_error_rate, 'bo-', 'MarkerSize', 8, 'DisplayName', 'IntDB');
    hold on;
    plot(influxdb_dur.duration, influxdb_error_rate, 'rs-', 'MarkerSize', 8, 'DisplayName', 'InfluxDB');
    xlabel('Test Duration (seconds)');
    ylabel('Error Rate (%)');
    title('Long-term Error Rate Trend');
    legend('Location', 'best');
    grid on;
    
    % Save chart
    saveas(gcf, fullfile(data_path, 'matlab_duration_analysis.png'));
    fprintf('✅ Duration analysis completed, chart saved\n');
else
    fprintf('❌ Duration test data file not found\n');
end

%% 3. Functional Endpoint Performance Comparison
fprintf('Analyzing functional endpoint performance...\n');

endpoint_file = fullfile(data_path, 'functional_endpoints.csv');
if exist(endpoint_file, 'file')
    endpoint_data = readtable(endpoint_file);
    
    % Get unique endpoints
    unique_endpoints = unique(endpoint_data.endpoint);
    
    % Create endpoint comparison chart
    figure('Name', 'Functional Endpoint Performance Comparison', 'Position', [200, 200, 1200, 800], 'Color', 'white');
    
    % Prepare data
    intdb_resp_times = [];
    influxdb_resp_times = [];
    intdb_qps_vals = [];
    influxdb_qps_vals = [];
    intdb_avail = [];
    influxdb_avail = [];
    endpoint_labels = {};
    
    for i = 1:length(unique_endpoints)
        ep = unique_endpoints{i};
        
        % IntDB data
        intdb_row = endpoint_data(strcmp(endpoint_data.endpoint, ep) & strcmp(endpoint_data.database, 'IntDB'), :);
        if ~isempty(intdb_row)
            intdb_resp_times(end+1) = intdb_row.response_time(1);
            intdb_qps_vals(end+1) = intdb_row.qps(1);
            intdb_avail(end+1) = intdb_row.availability(1);
            endpoint_labels{end+1} = ['IntDB' ep];
        end
        
        % InfluxDB data
        influxdb_row = endpoint_data(strcmp(endpoint_data.endpoint, ep) & strcmp(endpoint_data.database, 'InfluxDB'), :);
        if ~isempty(influxdb_row)
            influxdb_resp_times(end+1) = influxdb_row.response_time(1);
            influxdb_qps_vals(end+1) = influxdb_row.qps(1);
            influxdb_avail(end+1) = influxdb_row.availability(1);
            endpoint_labels{end+1} = ['InfluxDB' ep];
        end
    end
    
    % Subplot 1: Response Time Comparison
    subplot(2, 2, 1);
    x_pos = 1:length(endpoint_labels);
    all_resp_times = [intdb_resp_times, influxdb_resp_times];
    bar_colors = repmat([0 0.4470 0.7410], length(intdb_resp_times), 1);
    bar_colors = [bar_colors; repmat([0.8500 0.3250 0.0980], length(influxdb_resp_times), 1)];
    
    b1 = bar(x_pos, all_resp_times, 'FaceColor', 'flat');
    b1.CData = bar_colors;
    xlabel('Endpoint');
    ylabel('Response Time (ms)');
    title('Endpoint Response Time Comparison');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    grid on;
    
    % Subplot 2: QPS Comparison
    subplot(2, 2, 2);
    all_qps = [intdb_qps_vals, influxdb_qps_vals];
    b2 = bar(x_pos, all_qps, 'FaceColor', 'flat');
    b2.CData = bar_colors;
    xlabel('Endpoint');
    ylabel('QPS');
    title('Endpoint Throughput Comparison');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    grid on;
    
    % Subplot 3: Availability Comparison
    subplot(2, 2, 3);
    all_avail = [intdb_avail, influxdb_avail];
    b3 = bar(x_pos, all_avail, 'FaceColor', 'flat');
    b3.CData = bar_colors;
    xlabel('Endpoint');
    ylabel('Availability (%)');
    title('Endpoint Availability Comparison');
    set(gca, 'XTickLabel', endpoint_labels);
    xtickangle(45);
    ylim([99, 100.1]);
    grid on;
    
    % Subplot 4: Comprehensive Performance Radar Chart (Simplified)
    subplot(2, 2, 4);
    if ~isempty(intdb_resp_times) && ~isempty(influxdb_resp_times)
        % Calculate averages
        intdb_avg_resp = mean(intdb_resp_times);
        influxdb_avg_resp = mean(influxdb_resp_times);
        intdb_avg_qps = mean(intdb_qps_vals);
        influxdb_avg_qps = mean(influxdb_qps_vals);
        intdb_avg_avail = mean(intdb_avail);
        influxdb_avg_avail = mean(influxdb_avail);
        
        % Normalize data (0-1 range)
        max_resp = max(intdb_avg_resp, influxdb_avg_resp);
        max_qps = max(intdb_avg_qps, influxdb_avg_qps);
        
        intdb_norm = [1 - intdb_avg_resp/max_resp, intdb_avg_qps/max_qps, intdb_avg_avail/100];
        influxdb_norm = [1 - influxdb_avg_resp/max_resp, influxdb_avg_qps/max_qps, influxdb_avg_avail/100];
        
        categories = {'Response Time', 'QPS', 'Availability'};
        angles = linspace(0, 2*pi, length(categories)+1);
        
        % Draw radar chart
        polarplot(angles, [intdb_norm, intdb_norm(1)], 'bo-', 'LineWidth', 2, 'MarkerSize', 8);
        hold on;
        polarplot(angles, [influxdb_norm, influxdb_norm(1)], 'rs-', 'LineWidth', 2, 'MarkerSize', 8);
        
        % Set angle labels
        thetaticks(rad2deg(angles(1:end-1)));
        thetaticklabels(categories);
        title('Comprehensive Performance Radar Chart');
        legend('IntDB', 'InfluxDB', 'Location', 'best');
    end
    
    % Save chart
    saveas(gcf, fullfile(data_path, 'matlab_endpoint_analysis.png'));
    fprintf('✅ Functional endpoint analysis completed, chart saved\n');
else
    fprintf('❌ Functional endpoint test data file not found\n');
end

%% 4. Burst Load Test Analysis
fprintf('Analyzing burst load performance...\n');

burst_file = fullfile(data_path, 'burst_load.csv');
if exist(burst_file, 'file')
    burst_data = readtable(burst_file);
    
    % Separate data
    intdb_burst = burst_data(strcmp(burst_data.database, 'IntDB'), :);
    influxdb_burst = burst_data(strcmp(burst_data.database, 'InfluxDB'), :);
    
    % Create burst load analysis chart
    figure('Name', 'Burst Load Test Analysis', 'Position', [250, 250, 1000, 600], 'Color', 'white');
    
    phases = {'low', 'burst', 'recovery'};
    phase_labels = {'Low Load', 'Burst Load', 'Recovery Phase'};
    
    % Subplot 1: Response Time Comparison
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
    xlabel('Load Phase');
    ylabel('Response Time (ms)');
    title('Burst Load Response Time');
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % Subplot 2: QPS Comparison
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
    xlabel('Load Phase');
    ylabel('QPS');
    title('Burst Load Throughput');
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % Subplot 3: Time Series Chart
    subplot(2, 1, 2);
    plot(1:3, intdb_resp, 'bo-', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'IntDB Response Time');
    hold on;
    plot(1:3, influxdb_resp, 'rs-', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'InfluxDB Response Time');
    
    yyaxis right;
    plot(1:3, intdb_qps_burst, 'b^--', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'IntDB QPS');
    plot(1:3, influxdb_qps_burst, 'rd--', 'LineWidth', 2, 'MarkerSize', 8, 'DisplayName', 'InfluxDB QPS');
    ylabel('QPS');
    
    yyaxis left;
    ylabel('Response Time (ms)');
    xlabel('Load Phase');
    title('Burst Load Time Series Analysis');
    set(gca, 'XTick', 1:3);
    set(gca, 'XTickLabel', phase_labels);
    legend('Location', 'best');
    grid on;
    
    % Save chart
    saveas(gcf, fullfile(data_path, 'matlab_burst_analysis.png'));
    fprintf('✅ Burst load analysis completed, chart saved\n');
else
    fprintf('❌ Burst load test data file not found\n');
end

%% 5. Generate Performance Summary Statistics
fprintf('Generating performance summary statistics...\n');

% Create summary report
summary_file = fullfile(data_path, 'matlab_performance_summary.txt');
fid = fopen(summary_file, 'w');

fprintf(fid, 'IntDB vs InfluxDB Performance Test Summary Report\n');
fprintf(fid, '=================================================\n\n');
fprintf(fid, 'Report Generated: %s\n\n', datestr(now));

% Concurrency test summary
if exist('intdb_conc', 'var') && exist('influxdb_conc', 'var')
    fprintf(fid, '1. Concurrency Scalability Test Summary\n');
    fprintf(fid, '----------------------------------------\n');
    fprintf(fid, 'IntDB Average Response Time: %.2f ms\n', mean(intdb_conc.response_time));
    fprintf(fid, 'InfluxDB Average Response Time: %.2f ms\n', mean(influxdb_conc.response_time));
    fprintf(fid, 'IntDB Average QPS: %.1f\n', mean(intdb_conc.qps));
    fprintf(fid, 'InfluxDB Average QPS: %.1f\n', mean(influxdb_conc.qps));
    fprintf(fid, 'IntDB Best QPS: %.1f (Concurrency: %d)\n', max(intdb_conc.qps), intdb_conc.concurrency(intdb_conc.qps == max(intdb_conc.qps)));
    fprintf(fid, 'InfluxDB Best QPS: %.1f (Concurrency: %d)\n\n', max(influxdb_conc.qps), influxdb_conc.concurrency(influxdb_conc.qps == max(influxdb_conc.qps)));
end

% Duration test summary
if exist('intdb_dur', 'var') && exist('influxdb_dur', 'var')
    fprintf(fid, '2. Duration Test Summary\n');
    fprintf(fid, '------------------------\n');
    fprintf(fid, 'IntDB Long-term Average QPS: %.1f\n', mean(intdb_dur.qps));
    fprintf(fid, 'InfluxDB Long-term Average QPS: %.1f\n', mean(influxdb_dur.qps));
    fprintf(fid, 'IntDB Long-term Average Response Time: %.2f ms\n', mean(intdb_dur.response_time));
    fprintf(fid, 'InfluxDB Long-term Average Response Time: %.2f ms\n\n', mean(influxdb_dur.response_time));
end

fprintf(fid, '3. Generated Chart Files\n');
fprintf(fid, '------------------------\n');
fprintf(fid, '- matlab_concurrency_analysis.png\n');
fprintf(fid, '- matlab_duration_analysis.png\n');
fprintf(fid, '- matlab_endpoint_analysis.png\n');
fprintf(fid, '- matlab_burst_analysis.png\n\n');

fprintf(fid, 'Note: All charts have been saved in the test results directory\n');

fclose(fid);
fprintf('✅ Performance summary report saved: %s\n', summary_file);

%% Complete
fprintf('\n🎉 MATLAB performance analysis completed!\n');
fprintf('📊 Generated files:\n');
fprintf('   - matlab_concurrency_analysis.png\n');
fprintf('   - matlab_duration_analysis.png\n');
fprintf('   - matlab_endpoint_analysis.png\n');
fprintf('   - matlab_burst_analysis.png\n');
fprintf('   - matlab_performance_summary.txt\n');
fprintf('\nPlease check the chart files in %s directory\n', data_path); 