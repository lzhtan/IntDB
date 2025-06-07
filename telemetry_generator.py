#!/usr/bin/env python3
"""
IntDB Telemetry Data Generator
Sends telemetry data to IntDB every second with varying values
All data belongs to the same flow, only measurement values change
"""

import json
import time
import random
import urllib.request
import urllib.parse
from datetime import datetime, timezone

# Configuration
INTDB_URL = "http://localhost:2999/flows"
FIXED_FLOW_ID = "network_monitoring_flow"  # Fixed flow ID for all telemetry data

class TelemetryGenerator:
    def __init__(self, base_url, flow_id):
        self.base_url = base_url
        self.flow_id = flow_id
        self.measurement_count = 0
        # 固定的网络路径 - 4个交换机
        self.network_path = ["s1", "s2", "s3", "s4"]
        
    def generate_telemetry(self):
        """Generate telemetry data for fixed network path"""
        current_time = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        
        telemetry_data = []
        
        # 为固定路径上的每个交换机生成遥测数据
        for switch_id in self.network_path:
            delay_ns = random.randint(100, 800)  # 延迟 100-800 纳秒
            queue_util = round(random.uniform(0.05, 0.95), 2)  # 队列利用率 5%-95%
            
            telemetry_data.append({
                "switch_id": switch_id,
                "timestamp": current_time,
                "queue_util": queue_util,
                "delay_ns": delay_ns
            })
        
        return {
            "flow": {
                "flow_id": self.flow_id,
                "telemetry": telemetry_data
            }
        }
    
    def send_telemetry(self, data):
        """Send telemetry data to IntDB"""
        try:
            # Prepare the request
            json_data = json.dumps(data).encode('utf-8')
            req = urllib.request.Request(
                self.base_url,
                data=json_data,
                headers={'Content-Type': 'application/json'}
            )
            
            # Send the request
            with urllib.request.urlopen(req, timeout=5) as response:
                if response.status == 200:
                    timestamp = data["flow"]["telemetry"][0]["timestamp"]
                    self.measurement_count += 1
                    
                    if self.measurement_count == 1:
                        print(f"✓ [{timestamp}] Created flow {self.flow_id} - path: {' → '.join(self.network_path)}")
                    else:
                        print(f"✓ [{timestamp}] Measurement #{self.measurement_count} - ", end="")
                    
                    # 显示每个交换机的当前状态
                    for i, telemetry in enumerate(data["flow"]["telemetry"]):
                        switch_id = telemetry["switch_id"]
                        delay = telemetry["delay_ns"]
                        queue = telemetry["queue_util"]
                        if self.measurement_count > 1:
                            print(f"{switch_id}(q:{queue:.2f},d:{delay}ns)", end="")
                            if i < len(data["flow"]["telemetry"]) - 1:
                                print(" → ", end="")
                    
                    if self.measurement_count > 1:
                        print()  # 换行
                    
                    return True
                else:
                    response_text = response.read().decode('utf-8')
                    print(f"✗ HTTP {response.status}: {response_text}")
                    return False
                    
        except Exception as e:
            print(f"✗ Request failed: {e}")
            return False
    
    def check_connection(self):
        """Check if IntDB is running"""
        try:
            health_url = self.base_url.replace("/flows", "/health")
            with urllib.request.urlopen(health_url, timeout=5) as response:
                if response.status == 200:
                    return True
        except:
            pass
        return False
    
    def run(self):
        """Main loop to continuously send telemetry data"""
        print("=" * 60)
        print("  IntDB Telemetry Data Generator Started")
        print("=" * 60)
        print(f"Target: {self.base_url}")
        print(f"Flow ID: {self.flow_id} (FIXED - all data appends to same flow)")
        print(f"Interval: 1 second")
        print(f"Press Ctrl+C to stop")
        print()
        
        # Check connection
        print("Checking IntDB connection... ", end="", flush=True)
        if self.check_connection():
            print("✓ Connected")
        else:
            print("✗ Cannot connect to IntDB")
            print("Please ensure IntDB is running on localhost:2999")
            return
        
        print()
        
        try:
            while True:
                # Generate and send telemetry data
                telemetry_data = self.generate_telemetry()
                self.send_telemetry(telemetry_data)
                
                # Wait 1 second
                time.sleep(1)
                
        except KeyboardInterrupt:
            print(f"\n\nStopping telemetry generator...")
            print(f"Generated {self.measurement_count} telemetry measurements")

if __name__ == "__main__":
    generator = TelemetryGenerator(INTDB_URL, FIXED_FLOW_ID)
    generator.run() 