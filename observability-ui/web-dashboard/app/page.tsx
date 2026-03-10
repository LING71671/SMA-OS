"use client";

import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  MarkerType,
} from "reactflow";
import "reactflow/dist/style.css";
import { Activity, Zap, Server } from "lucide-react";
import { motion } from "framer-motion";

const initialNodes = [
  { id: "1", position: { x: 400, y: 50 }, data: { label: "Ingestion Pipeline (SLM)" }, type: "default", className: "custom-node active" },
  { id: "2", position: { x: 200, y: 200 }, data: { label: "Rule Fallback Analyzer" }, className: "custom-node" },
  { id: "3", position: { x: 600, y: 200 }, data: { label: "Orchestrator Manager (Go)" }, className: "custom-node active" },
  { id: "4", position: { x: 400, y: 350 }, data: { label: "Evaluator Critic" }, className: "custom-node rejected" },
  { id: "5", position: { x: 600, y: 500 }, data: { label: "Sandboxed Worker (Firecracker)" }, className: "custom-node pulsing" },
  { id: "6", position: { x: 200, y: 500 }, data: { label: "State Engine (Rust/Pg)" }, className: "custom-node" },
  { id: "7", position: { x: 800, y: 350 }, data: { label: "eBPF Fractal Gateway" }, className: "custom-node pulsing" },
];

const initialEdges = [
  { id: "e1-2", source: "1", target: "2", animated: true, stroke: "#6366f1" },
  { id: "e1-3", source: "1", target: "3", animated: true, stroke: "#10b981", markerEnd: { type: MarkerType.ArrowClosed } },
  { id: "e3-4", source: "3", target: "4", animated: true },
  { id: "e3-5", source: "3", target: "5", animated: true, stroke: "#10b981" },
  { id: "e4-3", source: "4", target: "3", animated: true, stroke: "#ef4444", label: "Versioned Reject", labelStyle: { fill: "#ef4444" } },
  { id: "e5-6", source: "5", target: "6", animated: true },
  { id: "e7-5", source: "7", target: "5", animated: true, stroke: "#eab308", label: "Nanosecond Intercept" },
];

export default function DagViewer() {
  const [nodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, onEdgesChange] = useEdgesState(initialEdges);

  return (
    <div style={{ width: "100vw", height: "100vh", position: "relative" }}>
      {/* Background Decor */}
      <div style={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, background: "radial-gradient(circle at 50% 50%, #1a1a2e 0%, #0a0a0f 100%)", zIndex: -1 }} />
      
      {/* Top Banner */}
      <motion.div 
        initial={{ y: -50, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.8 }}
        className="glass-panel"
        style={{ position: "absolute", top: 20, left: "50%", transform: "translateX(-50%)", zIndex: 10, display: "flex", gap: "20px", padding: "15px 30px", alignItems: "center" }}
      >
        <Zap color="#6366f1" />
        <h1 style={{ margin: 0, fontSize: "1.2rem", fontWeight: 800, letterSpacing: "2px" }} className="text-neon">
          SMA-OS OBSERVABILITY PLANE
        </h1>
        <Activity color="#10b981" className="pulsing" />
      </motion.div>

      {/* React Flow Canvas */}
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        attributionPosition="bottom-right"
      >
        <Background gap={20} size={1} color="#333" />
        <MiniMap nodeStrokeColor="#6366f1" nodeColor="#1a1a2e" maskColor="rgba(0,0,0,0.8)" style={{ background: '#0a0a0f', border: '1px solid #333' }} />
        <Controls style={{ display: 'flex', flexDirection: 'column', gap: '5px', background: '#1a1a2e', border: '1px solid #333', borderRadius: '8px', overflow: 'hidden' }} />
      </ReactFlow>

      {/* Side Panel: Live AI Context */}
      <motion.div 
        initial={{ x: 300, opacity: 0 }}
        animate={{ x: 0, opacity: 1 }}
        transition={{ delay: 0.5, duration: 0.8 }}
        className="glass-panel"
        style={{ position: "absolute", right: 20, top: 100, width: "320px", padding: "20px", display: "flex", flexDirection: "column", gap: "10px" }}
      >
        <h3 style={{ borderBottom: "1px solid #333", paddingBottom: "10px", margin: "0 0 10px 0", display: "flex", alignItems: "center", gap: "10px" }}>
          <Server size={18} /> LIVE EXECUTION LOG
        </h3>
        
        <div style={{ fontSize: "0.85rem", color: "#aaa", lineHeight: "1.5" }}>
          <p><span style={{color: "#10b981"}}>[SLM]</span> Confidence 99.2% - Route to Orchestrator</p>
          <p><span style={{color: "#6366f1"}}>[Manager]</span> Batching 50 Sub-tasks...</p>
          <p><span style={{color: "#ef4444"}}>[Evaluator]</span> Reject Task 42 (Schema Mismatch!) Rollback V2.</p>
          <p><span style={{color: "#eab308"}}>[eBPF]</span> SIGKILL intercepted syscall on Firecracker Pool A. Threat neutralized.</p>
        </div>
      </motion.div>
    </div>
  );
}
