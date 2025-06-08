#!/usr/bin/env python3
"""
IntDB vs InfluxDB 综合性能测试套件
包含：数据写入、查询性能测试、结果分析
"""

import json
import time
import random
import urllib.request
import urllib.parse
import statistics
from datetime import datetime, timezone, timedelta
import concurrent.futures
import threading
import argparse

# Configuration
INTDB_URL = "http://localhost:2999"
INFLUXDB_URL = "http://localhost:8086"
INFLUXDB_TOKEN = "UvasEX6JF0VA0EtZkMSwvVkYUluoNfyOmt7dfrEjA-15y2ECpYfPynB3QVVIfC685opfxS4HqMjP1abMpBWF4Q=="
INFLUXDB_ORG = "test-org"
INFLUXDB_BUCKET = "test-bucket"

class ComprehensivePerformanceTester:
    def __init__(self):
        self.intdb_success = 0
        self.influxdb_success = 0
        self.intdb_errors = 0
        self.influxdb_errors = 0
        self.lock = threading.Lock()
        
        # Test results
        self.query_results = {
            'intdb': {},
            'influxdb': {}
        }
        
        # Network topology
        self.spine_switches = ["spine-1", "spine-2", "spine-3", "spine-4"]
        self.leaf_switches = ["leaf-1", "leaf-2", "leaf-3", "leaf-4", 
                             "leaf-5", "leaf-6", "leaf-7", "leaf-8"]
        self.servers = [f"server-{i}" for i in range(1, 33)]  # 32 servers
        
    def generate_realistic_path(self):
        """Generate realistic spine-leaf network path"""
        # Typical data center path: server -> leaf -> spine -> leaf -> server
        src_server = random.choice(self.servers)
        dst_server = random.choice([s for s in self.servers if s != src_server])
        src_leaf = random.choice(self.leaf_switches)
        dst_leaf = random.choice([s for s in self.leaf_switches if s != src_leaf])
        spine = random.choice(self.spine_switches)
        
        return [src_server, src_leaf, spine, dst_leaf, dst_server]
    
    def generate_telemetry_batch(self, batch_size, start_time):
        """Generate a batch of telemetry data"""
        intdb_batch = []
        influxdb_batch = []
        
        for i in range(batch_size):
            # Generate unique flow ID
            flow_id = f"flow_{random.randint(100000, 999999)}_{i}"
            
            # Generate path
            path = self.generate_realistic_path()
            
            # Generate timestamp (distributed over last 24 hours)
            timestamp_offset = random.randint(0, 24 * 3600)  # seconds
            timestamp = start_time - timedelta(seconds=timestamp_offset)
            
            # Generate telemetry for each hop
            telemetry_data = []
            influx_lines = []
            
            total_delay = 0
            for hop_idx, switch_id in enumerate(path):
                delay_ns = random.randint(50, 1000)
                queue_util = round(random.uniform(0.01, 0.98), 3)
                jitter_ns = random.randint(1, 100)
                packet_loss = round(random.uniform(0.0, 0.05), 4)
                
                total_delay += delay_ns
                
                # IntDB format
                telemetry_data.append({
                    "switch_id": switch_id,
                    "hop_index": hop_idx,
                    "timestamp": timestamp.strftime("%Y-%m-%dT%H:%M:%SZ"),
                    "delay_ns": delay_ns,
                    "queue_util": queue_util,
                    "jitter_ns": jitter_ns,
                    "packet_loss": packet_loss
                })
                
                # InfluxDB Line Protocol format
                influx_line = (f"hop_metrics,flow_id={flow_id},switch_id={switch_id},"
                              f"hop_index={hop_idx} "
                              f"delay_ns={delay_ns},queue_util={queue_util},"
                              f"jitter_ns={jitter_ns},packet_loss={packet_loss} "
                              f"{int(timestamp.timestamp() * 1000000000)}")
                influx_lines.append(influx_line)
            
            # IntDB record
            intdb_record = {
                "flow": {
                    "flow_id": flow_id,
                    "path": path,
                    "total_delay_ns": total_delay,
                    "hop_count": len(path),
                    "telemetry": telemetry_data
                }
            }
            intdb_batch.append(intdb_record)
            
            # InfluxDB flow summary
            flow_summary = (f"flow_summary,flow_id={flow_id} "
                           f"total_delay_ns={total_delay},hop_count={len(path)},"
                           f"path=\"{' -> '.join(path)}\" "
                           f"{int(timestamp.timestamp() * 1000000000)}")
            influx_lines.append(flow_summary)
            influxdb_batch.extend(influx_lines)
        
        return intdb_batch, influxdb_batch
    
    def send_intdb_batch(self, batch):
        """Send batch to IntDB"""
        try:
            for record in batch:
                json_data = json.dumps(record).encode('utf-8')
                req = urllib.request.Request(
                    f"{INTDB_URL}/flows",
                    data=json_data,
                    headers={'Content-Type': 'application/json'}
                )
                
                with urllib.request.urlopen(req, timeout=10) as response:
                    if response.status == 200:
                        with self.lock:
                            self.intdb_success += 1
                    else:
                        with self.lock:
                            self.intdb_errors += 1
                            
        except Exception as e:
            with self.lock:
                self.intdb_errors += len(batch)
            print(f"IntDB batch error: {e}")
    
    def send_influxdb_batch(self, batch):
        """Send batch to InfluxDB"""
        try:
            # Join all lines with newline
            data = '\n'.join(batch).encode('utf-8')
            
            req = urllib.request.Request(
                f"{INFLUXDB_URL}/api/v2/write?org={INFLUXDB_ORG}&bucket={INFLUXDB_BUCKET}",
                data=data,
                headers={
                    'Authorization': f'Token {INFLUXDB_TOKEN}',
                    'Content-Type': 'text/plain; charset=utf-8'
                }
            )
            
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 204:  # InfluxDB returns 204 for successful writes
                    with self.lock:
                        self.influxdb_success += len([line for line in batch if line.startswith('hop_metrics')])
                else:
                    with self.lock:
                        self.influxdb_errors += len([line for line in batch if line.startswith('hop_metrics')])
                        
        except Exception as e:
            with self.lock:
                self.influxdb_errors += len([line for line in batch if line.startswith('hop_metrics')])
            print(f"InfluxDB batch error: {e}")
    
    def check_connections(self):
        """Check if both databases are accessible"""
        print("检查数据库连接...")
        
        # Check IntDB
        try:
            req = urllib.request.Request(f"{INTDB_URL}/health")
            with urllib.request.urlopen(req, timeout=5) as response:
                if response.status == 200:
                    print("✓ IntDB 连接正常")
                    intdb_ok = True
                else:
                    print("✗ IntDB 连接失败")
                    intdb_ok = False
        except Exception as e:
            print(f"✗ IntDB 连接失败: {e}")
            intdb_ok = False
        
        # Check InfluxDB
        try:
            req = urllib.request.Request(
                f"http://localhost:8086/ping",
                headers={'Authorization': f'Token {INFLUXDB_TOKEN}'}
            )
            with urllib.request.urlopen(req, timeout=5) as response:
                print("✓ InfluxDB 连接正常")
                influxdb_ok = True
        except Exception as e:
            print(f"✗ InfluxDB 连接失败: {e}")
            influxdb_ok = False
        
        return intdb_ok, influxdb_ok
    
    def run_data_generation(self, total_records=10000, batch_size=100):
        """Run data generation and insertion"""
        print("=" * 60)
        print("  数据生成和写入测试")
        print("=" * 60)
        print(f"目标记录数: {total_records:,}")
        print(f"批次大小: {batch_size}")
        print()
        
        # Check connections
        intdb_ok, influxdb_ok = self.check_connections()
        if not (intdb_ok and influxdb_ok):
            print("请修复连接问题后再继续。")
            return False
        
        print()
        print("开始数据生成和插入...")
        start_time = datetime.now(timezone.utc)
        generation_start = time.time()
        
        # Reset counters
        self.intdb_success = 0
        self.influxdb_success = 0
        self.intdb_errors = 0
        self.influxdb_errors = 0
        
        # Process in batches
        batches_total = (total_records + batch_size - 1) // batch_size
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
            for batch_num in range(batches_total):
                current_batch_size = min(batch_size, total_records - batch_num * batch_size)
                
                # Generate batch
                intdb_batch, influxdb_batch = self.generate_telemetry_batch(
                    current_batch_size, start_time
                )
                
                # Submit to both databases concurrently
                intdb_future = executor.submit(self.send_intdb_batch, intdb_batch)
                influxdb_future = executor.submit(self.send_influxdb_batch, influxdb_batch)
                
                # Wait for completion
                concurrent.futures.wait([intdb_future, influxdb_future])
                
                # Progress update
                completed = (batch_num + 1) * current_batch_size
                progress = (completed / total_records) * 100
                if total_records >= 10 and completed % (total_records // 10) == 0 or completed == total_records:
                    print(f"进度: {completed:,}/{total_records:,} ({progress:.1f}%) "
                          f"- IntDB: {self.intdb_success:,} 成功, {self.intdb_errors:,} 错误 "
                          f"- InfluxDB: {self.influxdb_success:,} 成功, {self.influxdb_errors:,} 错误")
        
        total_time = time.time() - generation_start
        
        print()
        print("=" * 60)
        print("  数据生成完成")
        print("=" * 60)
        print(f"总耗时: {total_time:.2f} 秒")
        print(f"写入速度: {total_records/total_time:.0f} 记录/秒")
        print()
        print("IntDB 结果:")
        print(f"  ✓ 成功: {self.intdb_success:,}")
        print(f"  ✗ 错误: {self.intdb_errors:,}")
        print()
        print("InfluxDB 结果:")
        print(f"  ✓ 成功: {self.influxdb_success:,}")
        print(f"  ✗ 错误: {self.influxdb_errors:,}")
        print()
        
        return True
    
    # =======================
    # 查询性能测试部分
    # =======================
    
    def measure_query_time(self, query_func, *args):
        """Measure execution time of a query function"""
        start_time = time.time()
        result = query_func(*args)
        end_time = time.time()
        return (end_time - start_time) * 1000, result  # Return time in milliseconds
    
    # Real IntDB Query Functions
    def intdb_path_reconstruction(self, flow_id):
        """Query 1: Path reconstruction - get complete path for a flow"""
        try:
            req = urllib.request.Request(f"{INTDB_URL}/flows/{flow_id}")
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode('utf-8'))
                    return data
                return None
        except Exception:
            return None
    
    def intdb_path_pattern_matching(self, pattern_switch):
        """Query 2: Path pattern matching - find flows containing specific switch"""
        try:
            req = urllib.request.Request(f"{INTDB_URL}/quick/through/{pattern_switch}")
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode('utf-8'))
                    return data
                return None
        except Exception:
            return None
    
    def intdb_path_aggregation(self, time_minutes=60):
        """Query 3: Path aggregation - get recent flows for analysis"""
        try:
            req = urllib.request.Request(f"{INTDB_URL}/quick/recent/{time_minutes}")
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    data = json.loads(response.read().decode('utf-8'))
                    return data
                return None
        except Exception:
            return None
    
    # InfluxDB Query Functions
    def influxdb_path_reconstruction(self, flow_id):
        """Query 1: Path reconstruction - requires multiple queries and joins"""
        try:
            flux_query = f'''
            from(bucket: "{INFLUXDB_BUCKET}")
              |> range(start: -24h)
              |> filter(fn: (r) => r._measurement == "flow_summary")
              |> filter(fn: (r) => r.flow_id == "{flow_id}")
              |> limit(n: 1)
            '''
            
            data = flux_query.encode('utf-8')
            req = urllib.request.Request(
                f"{INFLUXDB_URL}/api/v2/query?org={INFLUXDB_ORG}",
                data=data,
                headers={
                    'Authorization': f'Token {INFLUXDB_TOKEN}',
                    'Content-Type': 'application/vnd.flux',
                    'Accept': 'application/csv'
                }
            )
            
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    return response.read().decode('utf-8')
                return None
                
        except Exception:
            return None
    
    def influxdb_path_pattern_matching(self, pattern_switch):
        """Query 2: Path pattern matching - complex filtering"""
        try:
            flux_query = f'''
            from(bucket: "{INFLUXDB_BUCKET}")
              |> range(start: -24h)
              |> filter(fn: (r) => r._measurement == "hop_metrics")
              |> filter(fn: (r) => r.switch_id == "{pattern_switch}")
              |> group(columns: ["flow_id"])
              |> limit(n: 50)
            '''
            
            data = flux_query.encode('utf-8')
            req = urllib.request.Request(
                f"{INFLUXDB_URL}/api/v2/query?org={INFLUXDB_ORG}",
                data=data,
                headers={
                    'Authorization': f'Token {INFLUXDB_TOKEN}',
                    'Content-Type': 'application/vnd.flux',
                    'Accept': 'application/csv'
                }
            )
            
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    return response.read().decode('utf-8')
                return None
        except Exception:
            return None
    
    def influxdb_path_aggregation(self, time_minutes=60):
        """Query 3: Path aggregation - complex aggregation across tables"""
        try:
            flux_query = f'''
            from(bucket: "{INFLUXDB_BUCKET}")
              |> range(start: -{time_minutes}m)
              |> filter(fn: (r) => r._measurement == "flow_summary")
              |> filter(fn: (r) => r._field == "total_delay_ns")
              |> group(columns: ["path"])
              |> mean()
              |> limit(n: 50)
            '''
            
            data = flux_query.encode('utf-8')
            req = urllib.request.Request(
                f"{INFLUXDB_URL}/api/v2/query?org={INFLUXDB_ORG}",
                data=data,
                headers={
                    'Authorization': f'Token {INFLUXDB_TOKEN}',
                    'Content-Type': 'application/vnd.flux',
                    'Accept': 'application/csv'
                }
            )
            
            with urllib.request.urlopen(req, timeout=10) as response:
                if response.status == 200:
                    return response.read().decode('utf-8')
                return None
        except Exception:
            return None
    
    def run_query_test(self, query_name, intdb_func, influxdb_func, test_param=None, num_iterations=20):
        """Run a specific query test for both databases"""
        print(f"\n🔄 测试 {query_name}...")
        
        intdb_times = []
        influxdb_times = []
        intdb_successes = 0
        influxdb_successes = 0
        
        warmup_iterations = 3
        
        # Warmup
        print("  预热中...")
        for _ in range(warmup_iterations):
            if test_param:
                self.measure_query_time(intdb_func, test_param)
                self.measure_query_time(influxdb_func, test_param)
            else:
                self.measure_query_time(intdb_func)
                self.measure_query_time(influxdb_func)
        
        # Actual test
        print(f"  运行 {num_iterations} 次迭代...")
        for i in range(num_iterations):
            # Use different test parameters for variety
            if test_param and query_name == "路径重构":
                current_param = f"flow_{random.randint(100000, 999999)}_{random.randint(100, 119)}"
            else:
                current_param = test_param
            
            # Test IntDB
            if current_param:
                time_taken, result = self.measure_query_time(intdb_func, current_param)
            else:
                time_taken, result = self.measure_query_time(intdb_func)
            
            if result is not None:
                intdb_times.append(time_taken)
                intdb_successes += 1
            
            # Test InfluxDB
            if current_param:
                time_taken, result = self.measure_query_time(influxdb_func, current_param)
            else:
                time_taken, result = self.measure_query_time(influxdb_func)
            
            if result is not None:
                influxdb_times.append(time_taken)
                influxdb_successes += 1
            
            if (i + 1) % 5 == 0:
                print(f"    进度: {i + 1}/{num_iterations}")
        
        # Calculate statistics
        intdb_stats = self.calculate_stats(intdb_times) if intdb_times else None
        influxdb_stats = self.calculate_stats(influxdb_times) if influxdb_times else None
        
        self.query_results['intdb'][query_name] = {
            'times': intdb_times,
            'stats': intdb_stats,
            'success_rate': intdb_successes / num_iterations
        }
        
        self.query_results['influxdb'][query_name] = {
            'times': influxdb_times,
            'stats': influxdb_stats,
            'success_rate': influxdb_successes / num_iterations
        }
        
        print(f"  ✓ 完成. IntDB: {intdb_successes}/{num_iterations}, InfluxDB: {influxdb_successes}/{num_iterations}")
    
    def calculate_stats(self, times):
        """Calculate statistical metrics for query times"""
        if not times:
            return None
        
        return {
            'mean': statistics.mean(times),
            'median': statistics.median(times),
            'min': min(times),
            'max': max(times),
            'std_dev': statistics.stdev(times) if len(times) > 1 else 0,
            'p95': sorted(times)[int(len(times) * 0.95)] if len(times) > 1 else times[0],
            'p99': sorted(times)[int(len(times) * 0.99)] if len(times) > 1 else times[0]
        }
    
    def run_query_performance_test(self, num_iterations=20):
        """Run all query performance tests"""
        print("=" * 70)
        print("  查询性能测试: IntDB vs InfluxDB")
        print("=" * 70)
        print(f"每个测试迭代次数: {num_iterations}")
        print()
        
        # Test 1: Path Reconstruction
        test_flow_id = f"flow_{random.randint(100000, 999999)}_{random.randint(100, 119)}"
        self.run_query_test(
            "路径重构",
            self.intdb_path_reconstruction,
            self.influxdb_path_reconstruction,
            test_flow_id,
            num_iterations
        )
        
        # Test 2: Path Pattern Matching
        test_switch = random.choice(self.spine_switches + self.leaf_switches)
        self.run_query_test(
            "路径模式匹配",
            self.intdb_path_pattern_matching,
            self.influxdb_path_pattern_matching,
            test_switch,
            num_iterations
        )
        
        # Test 3: Path Aggregation
        self.run_query_test(
            "路径聚合",
            self.intdb_path_aggregation,
            self.influxdb_path_aggregation,
            60,
            num_iterations
        )
    
    # =======================
    # 结果分析部分
    # =======================
    
    def print_query_results(self):
        """Print comprehensive query test results"""
        print("\n" + "=" * 70)
        print("  查询性能测试结果")
        print("=" * 70)
        
        for query_name in self.query_results['intdb'].keys():
            print(f"\n📊 {query_name.upper()}")
            print("-" * 60)
            
            intdb_result = self.query_results['intdb'][query_name]
            influxdb_result = self.query_results['influxdb'][query_name]
            
            print(f"{'指标':<20} {'IntDB':<15} {'InfluxDB':<15} {'改进率':<15}")
            print("-" * 70)
            
            if intdb_result['stats'] and influxdb_result['stats']:
                intdb_stats = intdb_result['stats']
                influxdb_stats = influxdb_result['stats']
                
                # Calculate improvements (positive means IntDB is faster)
                mean_improvement = ((influxdb_stats['mean'] - intdb_stats['mean']) / influxdb_stats['mean']) * 100
                median_improvement = ((influxdb_stats['median'] - intdb_stats['median']) / influxdb_stats['median']) * 100
                p95_improvement = ((influxdb_stats['p95'] - intdb_stats['p95']) / influxdb_stats['p95']) * 100
                
                print(f"{'平均响应(ms)':<20} {intdb_stats['mean']:<15.2f} {influxdb_stats['mean']:<15.2f} {mean_improvement:>+.1f}%")
                print(f"{'中位数(ms)':<20} {intdb_stats['median']:<15.2f} {influxdb_stats['median']:<15.2f} {median_improvement:>+.1f}%")
                print(f"{'最小值(ms)':<20} {intdb_stats['min']:<15.2f} {influxdb_stats['min']:<15.2f}")
                print(f"{'最大值(ms)':<20} {intdb_stats['max']:<15.2f} {influxdb_stats['max']:<15.2f}")
                print(f"{'P95(ms)':<20} {intdb_stats['p95']:<15.2f} {influxdb_stats['p95']:<15.2f} {p95_improvement:>+.1f}%")
                print(f"{'标准差(ms)':<20} {intdb_stats['std_dev']:<15.2f} {influxdb_stats['std_dev']:<15.2f}")
            
            print(f"{'成功率':<20} {intdb_result['success_rate']*100:<15.1f}% {influxdb_result['success_rate']*100:<15.1f}%")
    
    def generate_summary_report(self):
        """Generate a comprehensive summary report"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Save detailed results
        results_filename = f"performance_test_results_{timestamp}.json"
        with open(results_filename, 'w') as f:
            json.dump({
                'write_performance': {
                    'intdb_success': self.intdb_success,
                    'intdb_errors': self.intdb_errors,
                    'influxdb_success': self.influxdb_success,
                    'influxdb_errors': self.influxdb_errors
                },
                'query_performance': self.query_results
            }, f, indent=2, ensure_ascii=False)
        
        # Generate markdown report
        report_filename = f"performance_analysis_report_{timestamp}.md"
        with open(report_filename, 'w', encoding='utf-8') as f:
            f.write("# IntDB vs InfluxDB 性能测试报告\n\n")
            f.write(f"**测试时间**: {datetime.now().strftime('%Y年%m月%d日 %H:%M:%S')}\n\n")
            
            f.write("## 数据写入性能\n\n")
            f.write("| 数据库 | 成功记录数 | 失败记录数 | 成功率 |\n")
            f.write("|--------|------------|------------|--------|\n")
            total_intdb = self.intdb_success + self.intdb_errors
            total_influxdb = self.influxdb_success + self.influxdb_errors
            if total_intdb > 0 and total_influxdb > 0:
                f.write(f"| IntDB | {self.intdb_success:,} | {self.intdb_errors:,} | {self.intdb_success/total_intdb*100:.1f}% |\n")
                f.write(f"| InfluxDB | {self.influxdb_success:,} | {self.influxdb_errors:,} | {self.influxdb_success/total_influxdb*100:.1f}% |\n\n")
            
            f.write("## 查询性能对比\n\n")
            for query_name in self.query_results['intdb'].keys():
                f.write(f"### {query_name}\n\n")
                intdb_result = self.query_results['intdb'][query_name]
                influxdb_result = self.query_results['influxdb'][query_name]
                
                if intdb_result['stats'] and influxdb_result['stats']:
                    intdb_stats = intdb_result['stats']
                    influxdb_stats = influxdb_result['stats']
                    mean_improvement = ((influxdb_stats['mean'] - intdb_stats['mean']) / influxdb_stats['mean']) * 100
                    
                    f.write("| 指标 | IntDB | InfluxDB | 改进率 |\n")
                    f.write("|------|-------|----------|--------|\n")
                    f.write(f"| 平均响应时间(ms) | {intdb_stats['mean']:.2f} | {influxdb_stats['mean']:.2f} | {mean_improvement:+.1f}% |\n")
                    f.write(f"| P95响应时间(ms) | {intdb_stats['p95']:.2f} | {influxdb_stats['p95']:.2f} | - |\n")
                    f.write(f"| 成功率 | {intdb_result['success_rate']*100:.1f}% | {influxdb_result['success_rate']*100:.1f}% | - |\n\n")
        
        print(f"\n💾 详细结果保存至: {results_filename}")
        print(f"📄 分析报告保存至: {report_filename}")
    
    def run_comprehensive_test(self, data_records=10000, query_iterations=20):
        """Run the complete test suite"""
        print("🚀 启动 IntDB vs InfluxDB 综合性能测试")
        print("=" * 70)
        
        # Step 1: Data Generation and Insertion
        print("📊 第一步: 数据写入性能测试")
        success = self.run_data_generation(total_records=data_records)
        if not success:
            print("❌ 数据写入测试失败，终止测试")
            return
        
        # Step 2: Query Performance Testing
        print("\n🔍 第二步: 查询性能测试")
        self.run_query_performance_test(num_iterations=query_iterations)
        
        # Step 3: Results Analysis
        print("\n📈 第三步: 结果分析")
        self.print_query_results()
        self.generate_summary_report()
        
        print("\n✅ 综合性能测试完成！")

def main():
    parser = argparse.ArgumentParser(description='IntDB vs InfluxDB 综合性能测试')
    parser.add_argument('--data-only', action='store_true', help='仅运行数据写入测试')
    parser.add_argument('--query-only', action='store_true', help='仅运行查询性能测试')
    parser.add_argument('--records', type=int, default=10000, help='写入的记录数量')
    parser.add_argument('--iterations', type=int, default=20, help='查询测试迭代次数')
    
    args = parser.parse_args()
    
    tester = ComprehensivePerformanceTester()
    
    if args.data_only:
        tester.run_data_generation(total_records=args.records)
    elif args.query_only:
        tester.run_query_performance_test(num_iterations=args.iterations)
    else:
        tester.run_comprehensive_test(data_records=args.records, query_iterations=args.iterations)

if __name__ == "__main__":
    main() 