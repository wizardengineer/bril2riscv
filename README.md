# Bril to RISC-V Compiler

## Side Note

Working on compilers is brutal, yet extremely rewarding. It feels like a dark-souls game. There's no other feeling, almost like a meta-level experience. 

Current Game Plan:
```
   parse JSON → flat IR → CFG → SSA
     ↓
   [1] CCP             // constant‐fold + branch elimination
   [2] Liveness + DCE  // kill any defs that aren’t used
     ↓
   break SSA 
     ↓
   instruction selection [per op → RISC-V template (MachineIR)]
     ↓
   build live‐intervals 
     ↓
   linear-scan register allocation
     ↓
   emit RISC-V text
```
