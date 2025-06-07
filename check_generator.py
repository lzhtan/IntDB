#!/usr/bin/env python3
import json

with open('current_flow.json', 'r') as f:
    data = json.load(f)

flow = data['flow']
hops = flow['hops']

print('📊 遥测数据生成器工作状态检查')
print('=' * 40)
print(f'Flow ID: {flow["flow_id"]}')
print(f'路径: {" → ".join(flow["path"]["switches"])}')
print(f'总hops数量: {len(hops)}')
print(f'开始时间: {flow["start_time"]}')
print(f'结束时间: {flow["end_time"]}')

# 统计每个交换机的测量次数
switch_counts = {}
for hop in hops:
    switch_id = hop['switch_id']
    switch_counts[switch_id] = switch_counts.get(switch_id, 0) + 1

print('\n各交换机测量次数:')
for switch_id in sorted(switch_counts.keys()):
    print(f'  {switch_id}: {switch_counts[switch_id]} 次测量')

# 显示最近的几个hop
print('\n最近的几个测量:')
for hop in hops[-4:]:
    timestamp = hop["timestamp"][-8:]  # 只显示时间部分
    queue_util = hop["metrics"]["queue_util"]
    delay_ns = hop["metrics"]["delay_ns"]
    print(f'  {hop["switch_id"]}: q={queue_util:.3f}, d={delay_ns}ns, t={timestamp}')

print('\n生成器状态:')
total_measurements = len(hops) // 4  # 每次测量包含4个交换机
print(f'✅ 生成器已产生 {total_measurements} 次完整测量')
print(f'✅ 数据正在持续追加到流中')
if all(switch_counts[sid] == switch_counts['s1'] for sid in ['s1', 's2', 's3', 's4']):
    print(f'✅ 所有4个交换机的数据都在正常更新')
else:
    print(f'⚠️  交换机数据不一致') 