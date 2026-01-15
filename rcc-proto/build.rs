syntax = "proto3";
package rcc;

service Coordinator {
    rpc RegisterExecutor (ExecutorInfo) returns (RegisterResponse);
    rpc Heartbeat (ExecutorStatus) returns (HeartbeatAck);
}

message ExecutorInfo {
    string id = 1;
    string address = 2;
    int32 max_memory_mb = 3;
}

message RegisterResponse {
    bool success = 1;
}

message ExecutorStatus {
    string id = 1;
    int32 active_instances = 2;
    float cpu_usage = 3;
}

message HeartbeatAck {}
