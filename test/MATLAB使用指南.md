# IntDB性能测试 MATLAB 分析指南

## 概述
这套MATLAB脚本用于分析IntDB与InfluxDB的性能测试结果，生成专业的性能对比图表。

## 文件说明

### 1. `matlab_performance_analysis.m` - 完整版分析脚本
**功能**：
- 并发扩展性分析（6个子图）
- 持续时间扩展性分析（4个子图）
- 功能端点性能对比（4个子图，包含雷达图）
- 突发负载测试分析（3个子图）
- 自动生成性能摘要报告

**生成图表**：
- `matlab_concurrency_analysis.png` - 并发扩展性分析
- `matlab_duration_analysis.png` - 持续时间分析
- `matlab_endpoint_analysis.png` - 端点性能对比
- `matlab_burst_analysis.png` - 突发负载分析
- `matlab_performance_summary.txt` - 文字摘要报告

### 2. `simple_matlab_analysis.m` - 简化版分析脚本
**功能**：
- 基本性能对比（4个子图）
- 控制台输出数值摘要
- 快速生成核心对比图

**生成图表**：
- `simple_performance_comparison.png` - 基本性能对比图

## 使用方法

### 步骤1：准备环境
```matlab
% 确保MATLAB已安装，版本建议2018b以上
% 确保有以下工具箱（可选）：
% - Statistics and Machine Learning Toolbox
% - Signal Processing Toolbox
```

### 步骤2：修改数据路径
```matlab
% 在脚本开头修改数据路径
data_path = 'test/performance_results_20250606_220608/';  % 修改为您的实际路径
```

### 步骤3：运行脚本
**完整分析**：
```matlab
>> matlab_performance_analysis
```

**快速分析**：
```matlab
>> simple_matlab_analysis
```

## 图表类型说明

### 1. 并发扩展性分析
- **响应时间扩展性**：双对数坐标，展示系统在不同并发下的响应时间变化
- **吞吐量扩展性**：半对数坐标，展示QPS随并发数的变化
- **可用性稳定性**：展示系统在高并发下的稳定性
- **错误率对比**：对数坐标展示失败事务数
- **延迟峰值对比**：双对数坐标展示最大延迟
- **并发效率对比**：双对数坐标展示每并发的QPS效率

### 2. 持续时间分析
- **长期QPS稳定性**：展示系统长期运行的吞吐量稳定性
- **长期响应时间稳定性**：展示响应时间的长期表现
- **总处理能力对比**：展示累计事务处理能力
- **长期错误率趋势**：展示系统长期运行的可靠性

### 3. 端点性能对比
- **端点响应时间对比**：柱状图对比不同API端点的响应时间
- **端点吞吐量对比**：柱状图对比不同端点的QPS
- **端点可用性对比**：柱状图对比不同端点的可用性
- **综合性能雷达图**：极坐标图展示综合性能对比

### 4. 突发负载分析
- **突发负载响应时间**：展示系统在负载突变时的响应时间表现
- **突发负载吞吐量**：展示系统在负载突变时的QPS表现
- **时间序列分析**：双Y轴图展示响应时间和QPS的时间序列变化

## 数据要求

脚本期望的CSV文件格式：

### `concurrency_scaling.csv`
```csv
concurrency,database,transactions,availability,response_time,qps,failed_transactions,max_latency
1,IntDB,16207,100.00,0.17,1076.16,0,190.00
1,InfluxDB,16207,100.00,0.14,1020.59,0,90.00
...
```

### `duration_scaling.csv`
```csv
duration,database,transactions,availability,response_time,qps,failed_transactions,max_latency
10,IntDB,14546,100.00,8.87,1391.96,0,6710.00
10,InfluxDB,1294,100.00,111.59,119.04,0,6720.00
...
```

### `functional_endpoints.csv`
```csv
endpoint,database,transactions,availability,response_time,qps,failed_transactions,max_latency
/health,IntDB,16246,99.92,6.07,530.92,13,8420.00
/ping,InfluxDB,16257,99.91,3.70,526.29,15,6720.00
...
```

### `burst_load.csv`
```csv
phase,database,transactions,availability,response_time,qps,failed_transactions,max_latency
low,IntDB,15572,100.00,1.84,742.58,0,6900.00
burst,IntDB,16307,100.00,21.61,781.74,0,7950.00
...
```

## 自定义选项

### 修改颜色主题
```matlab
% 在脚本开头修改默认颜色
set(groot, 'defaultAxesColorOrder', [
    0 0.4470 0.7410;      % 蓝色 - IntDB
    0.8500 0.3250 0.0980; % 红色 - InfluxDB
    0.9290 0.6940 0.1250  % 黄色 - 其他
]);
```

### 修改图表大小
```matlab
% 修改figure位置和大小
figure('Position', [x, y, width, height]);
```

### 保存不同格式
```matlab
% 保存为不同格式
saveas(gcf, 'filename.png');  % PNG格式
saveas(gcf, 'filename.pdf');  % PDF格式
saveas(gcf, 'filename.fig');  % MATLAB格式
print(gcf, 'filename.eps', '-depsc'); % EPS格式
```

## 故障排除

### 常见错误及解决方案

1. **文件路径错误**
   ```
   错误：Unable to read file 'xxx.csv'
   解决：检查并修改data_path变量
   ```

2. **数据格式错误**
   ```
   错误：Unrecognized table variable name
   解决：检查CSV文件列名是否正确
   ```

3. **图表显示问题**
   ```
   错误：图表显示不完整
   解决：调整figure大小或subplot布局
   ```

4. **中文显示问题**
   ```
   错误：中文显示为方框
   解决：安装中文字体或修改字体设置
   ```

## 输出示例

运行完整脚本后，您将看到：

```
正在分析并发扩展性...
✅ 并发扩展性分析完成，图表已保存
正在分析持续时间扩展性...
✅ 持续时间分析完成，图表已保存
正在分析功能端点性能...
✅ 功能端点分析完成，图表已保存
正在分析突发负载性能...
✅ 突发负载分析完成，图表已保存
正在生成性能摘要统计...
✅ 性能摘要报告已保存

🎉 MATLAB性能分析完成！
📊 生成的文件:
   - matlab_concurrency_analysis.png
   - matlab_duration_analysis.png
   - matlab_endpoint_analysis.png
   - matlab_burst_analysis.png
   - matlab_performance_summary.txt

请检查 test/performance_results_20250606_220608/ 目录中的图表文件
```

## 论文用图建议

生成的图表已针对学术论文进行优化：
- 高分辨率（300 DPI）
- 专业配色方案
- 清晰的标签和图例
- 适合黑白打印的线型和标记

建议用于论文的图表：
1. `matlab_concurrency_analysis.png` - 用于展示系统扩展性
2. `simple_performance_comparison.png` - 用于快速对比概览
3. `matlab_burst_analysis.png` - 用于展示系统稳定性 