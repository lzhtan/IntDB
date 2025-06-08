# IntDB vs InfluxDB 性能测试报告

## 测试时间
2025年 6月 6日 星期五 22时36分08秒 KST

## 测试环境
- 操作系统: Darwin 22.6.0
- IntDB URL: http://127.0.0.1:3000
- InfluxDB URL: http://127.0.0.1:8086

## 测试结果文件
1. 并发数递增测试: `concurrency_scaling.csv`
2. 持续时间测试: `duration_scaling.csv`
3. 功能端点测试: `functional_endpoints.csv`
4. 混合负载测试: `mixed_workload.csv`
5. 长期稳定性测试: `long_term_stability.csv`
6. 突发负载测试: `burst_load.csv`

## 快速分析脚本
使用以下Python脚本分析结果:

```python
import pandas as pd
import matplotlib.pyplot as plt

# 读取并发测试结果
df = pd.read_csv('concurrency_scaling.csv')
intdb_data = df[df['database'] == 'IntDB']
influxdb_data = df[df['database'] == 'InfluxDB']

# 绘制响应时间对比
plt.figure(figsize=(12, 8))
plt.subplot(2, 2, 1)
plt.plot(intdb_data['concurrency'], intdb_data['response_time'], 'b-o', label='IntDB')
plt.plot(influxdb_data['concurrency'], influxdb_data['response_time'], 'r-s', label='InfluxDB')
plt.xlabel('并发数')
plt.ylabel('响应时间 (ms)')
plt.title('响应时间 vs 并发数')
plt.legend()

plt.subplot(2, 2, 2)
plt.plot(intdb_data['concurrency'], intdb_data['qps'], 'b-o', label='IntDB')
plt.plot(influxdb_data['concurrency'], influxdb_data['qps'], 'r-s', label='InfluxDB')
plt.xlabel('并发数')
plt.ylabel('QPS')
plt.title('吞吐量 vs 并发数')
plt.legend()

plt.tight_layout()
plt.savefig('performance_comparison.png')
plt.show()
```
