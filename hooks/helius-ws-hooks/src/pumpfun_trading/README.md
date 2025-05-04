### Trading Framework

#### Thread 1
* pop payload from thread 1 message queue
* Process payload data in a loop
* Push buy instructions to thread 2 message queue

#### Thread 2:
* Maintain trading instruction queue, support multi-threaded execution of buy and sell subtasks
* Must not crash
* After executing buy instruction, write completion status to thread 1 message queue

#### Thread 3:
* Timer
* Receive initial event, wait for specified time, then send instruction to Thread 2

Trading Workflow Diagram
```mermaid
graph LR
    subgraph Thread 1
        A[Pop Payload] --> B{Process Payload};
        B --> C[Push Buy Instruction];
    end
    subgraph Thread 2
        D[Trading Instruction Queue] --> E{Execute Buy/Sell};
        E --> F[Write Completion Status];
    end
    subgraph Thread 3
        G[Timer] --> H[Send Instruction to Thread 2];
    end

    C --> D;
    F --> A;
    H --> D;
```
