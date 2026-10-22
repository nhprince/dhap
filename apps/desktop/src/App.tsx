import React, { useState, useRef } from 'react';
import { Project, GuideStep, Annotation, Redaction, LayoutMode } from './types';

// Sample initial project
const initialProject: Project = {
  id: 'proj_demo_001',
  title: 'Deploying Service with Dhap',
  description: 'A step-by-step verified workflow created with Dhap.',
  schema_version: 1,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
  sources: [
    {
      id: 'media_001',
      filename: 'deployment_session.mp4',
      duration_ms: 60000,
      width: 1280,
      height: 720,
      fps: 30,
      format: 'video/mp4',
    },
  ],
  frames: [
    {
      id: 'frame_001',
      timestamp_ms: 1500,
      relative_path: 'frames/frame_001.png',
      width: 800,
      height: 480,
      annotations: [
        {
          id: 'anno_1',
          kind: { type: 'CalloutBadge', data: { number: 1 } },
          rect: { x: 120, y: 140, width: 36, height: 36 },
          color_hex: '#F97316',
        },
      ],
      redactions: [
        {
          id: 'redact_1',
          rect: { x: 40, y: 35, width: 160, height: 28 },
          style: { style: 'Solid', params: { color_hex: '#000000' } },
        },
      ],
    },
    {
      id: 'frame_002',
      timestamp_ms: 8500,
      relative_path: 'frames/frame_002.png',
      width: 800,
      height: 480,
      annotations: [
        {
          id: 'anno_2',
          kind: { type: 'Rectangle' },
          rect: { x: 200, y: 160, width: 280, height: 140 },
          color_hex: '#22C55E',
        },
      ],
      redactions: [],
    },
  ],
  steps: [
    {
      id: 'step_001',
      order: 1,
      title: 'Authenticate to Gateway',
      instruction: 'Open terminal and execute deployment login with your credentials.',
      expected_result: 'Logged in successfully to production cluster.',
      notes: 'Make sure your VPN connection is active before proceeding.',
      frame_ids: ['frame_001'],
      source_timestamp_ms: 1500,
      status: 'reviewed',
    },
    {
      id: 'step_002',
      order: 2,
      title: 'Verify Cluster Healthcheck',
      instruction: 'Check the service dashboard to ensure all status probes return 200 OK.',
      expected_result: 'All nodes show green status indicator.',
      frame_ids: ['frame_002'],
      source_timestamp_ms: 8500,
      status: 'draft',
    },
  ],
  export_settings: {
    default_format: 'markdown,html',
    bake_redactions: true,
    include_timestamps: true,
    output_dir: 'dist_docs',
  },
};

export default function App() {
  const [project, setProject] = useState<Project>(initialProject);
  const [activeStepId, setActiveStepId] = useState<string>(project.steps[0].id);
  const [layout, setLayout] = useState<LayoutMode>('guide');
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [activeTool, setActiveTool] = useState<'select' | 'callout' | 'rectangle' | 'redact'>('select');
  const [showExportModal, setShowExportModal] = useState<boolean>(false);
  const [selectedTemplate, setSelectedTemplate] = useState<'standard' | 'sop' | 'classroom' | 'github'>('standard');
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);

  // Recording State
  const [showRecordModal, setShowRecordModal] = useState<boolean>(false);
  const [isRecording, setIsRecording] = useState<boolean>(false);
  const [recordingSec, setRecordingSec] = useState<number>(0);
  const [markers, setMarkers] = useState<Array<{ timestampMs: number; label: string }>>([]);
  const recordingTimerRef = useRef<number | null>(null);

  // AI Assist State
  const [aiGenerating, setAiGenerating] = useState<boolean>(false);
  const [aiStatus, setAiStatus] = useState<string | null>(null);

  const canvasRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const activeStep = project.steps.find((s) => s.id === activeStepId) || project.steps[0];
  const activeFrame = activeStep.frame_ids.length > 0
    ? project.frames.find((f) => f.id === activeStep.frame_ids[0])
    : null;

  const toggleTheme = () => {
    const next = theme === 'dark' ? 'light' : 'dark';
    setTheme(next);
    document.documentElement.setAttribute('data-theme', next);
  };

  const startLiveRecording = () => {
    setShowRecordModal(true);
    setIsRecording(true);
    setRecordingSec(0);
    setMarkers([]);
    if (recordingTimerRef.current) clearInterval(recordingTimerRef.current);
    recordingTimerRef.current = window.setInterval(() => {
      setRecordingSec((s) => s + 1);
    }, 1000);
  };

  const markRecordingStep = () => {
    const elapsedMs = recordingSec * 1000;
    const count = markers.length + 1;
    const label = `Action ${count}`;
    setMarkers((prev) => [...prev, { timestampMs: elapsedMs, label }]);
  };

  const stopAndFinalizeRecording = () => {
    if (recordingTimerRef.current) {
      clearInterval(recordingTimerRef.current);
      recordingTimerRef.current = null;
    }
    setIsRecording(false);

    if (markers.length > 0) {
      const generatedFrames = markers.map((m, idx) => ({
        id: `frame_rec_${idx + 1}_${Date.now()}`,
        timestamp_ms: m.timestampMs,
        relative_path: `frames/rec_frame_${idx + 1}.png`,
        width: 1280,
        height: 720,
        annotations: [],
        redactions: [],
      }));

      const generatedSteps: GuideStep[] = markers.map((m, idx) => ({
        id: `step_rec_${idx + 1}_${Date.now()}`,
        order: idx + 1,
        title: `Step ${idx + 1}: ${m.label}`,
        instruction: `Perform the action recorded at ${(m.timestampMs / 1000).toFixed(1)}s in the workflow.`,
        expected_result: `Verified state for step ${idx + 1}.`,
        frame_ids: [generatedFrames[idx].id],
        source_timestamp_ms: m.timestampMs,
        status: 'draft',
      }));

      setProject((prev) => ({
        ...prev,
        frames: [...prev.frames, ...generatedFrames],
        steps: generatedSteps,
      }));
      setActiveStepId(generatedSteps[0].id);
    }

    setShowRecordModal(false);
  };

  const handleSaveProject = () => {
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(project, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `${project.title.toLowerCase().replace(/[^a-z0-9]/g, '_')}.dhap.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  };

  const handleOpenProject = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const loaded = JSON.parse(event.target?.result as string);
        if (loaded.schema_version && loaded.steps) {
          setProject(loaded);
          if (loaded.steps.length > 0) {
            setActiveStepId(loaded.steps[0].id);
          }
        }
      } catch (err) {
        alert('Invalid Dhap project file: ' + err);
      }
    };
    reader.readAsText(file);
  };

  const handleAiAssist = () => {
    setAiGenerating(true);
    setAiStatus('Running local Privacy Pre-flight token scan...');

    setTimeout(() => {
      setAiStatus('Generating concise title & expected outcome...');
      setTimeout(() => {
        const refinedTitle = activeStep.title.replace(/^Step \d+:\s*/, '');
        handleUpdateStep({
          title: `Configure ${refinedTitle || 'Component'}`,
          expected_result: 'Operation completes with zero errors and system status turns green.',
        });
        setAiGenerating(false);
        setAiStatus(null);
      }, 700);
    }, 500);
  };


  const handleUpdateStep = (updated: Partial<GuideStep>) => {
    setProject((prev) => ({
      ...prev,
      steps: prev.steps.map((s) => (s.id === activeStep.id ? { ...s, ...updated } : s)),
    }));
  };

  const handleAddStep = () => {
    const newId = `step_${Date.now()}`;
    const newStep: GuideStep = {
      id: newId,
      order: project.steps.length ? project.steps.length + 1 : 1,
      title: 'New Guide Step',
      instruction: 'Describe what action the user must perform.',
      frame_ids: activeFrame ? [activeFrame.id] : [],
      status: 'draft',
    };
    setProject((prev) => ({
      ...prev,
      steps: [...prev.steps, newStep],
    }));
    setActiveStepId(newId);
  };

  const handleDeleteStep = (id: string) => {
    if (project.steps.length <= 1) return;
    setProject((prev) => ({
      ...prev,
      steps: prev.steps.filter((s) => s.id !== id),
    }));
    if (activeStepId === id) {
      setActiveStepId(project.steps[0].id);
    }
  };

  const handleMoveStepUp = (id: string, e?: React.MouseEvent) => {
    e?.stopPropagation();
    const idx = project.steps.findIndex((s) => s.id === id);
    if (idx <= 0) return;
    const nextSteps = [...project.steps];
    const temp = nextSteps[idx];
    nextSteps[idx] = nextSteps[idx - 1];
    nextSteps[idx - 1] = temp;
    nextSteps.forEach((s, i) => {
      s.order = i + 1;
    });
    setProject((prev) => ({ ...prev, steps: nextSteps }));
  };

  const handleMoveStepDown = (id: string, e?: React.MouseEvent) => {
    e?.stopPropagation();
    const idx = project.steps.findIndex((s) => s.id === id);
    if (idx < 0 || idx >= project.steps.length - 1) return;
    const nextSteps = [...project.steps];
    const temp = nextSteps[idx];
    nextSteps[idx] = nextSteps[idx + 1];
    nextSteps[idx + 1] = temp;
    nextSteps.forEach((s, i) => {
      s.order = i + 1;
    });
    setProject((prev) => ({ ...prev, steps: nextSteps }));
  };

  const detectedSecrets = project.steps.flatMap((step, idx) => {
    const text = `${step.title} ${step.instruction} ${step.expected_result || ''} ${step.notes || ''}`;
    const warnings: string[] = [];
    if (/ghp_[a-zA-Z0-9]{20,}/.test(text) || /github_pat_[a-zA-Z0-9_]{20,}/.test(text)) {
      warnings.push(`Step ${idx + 1}: GitHub Personal Access Token`);
    }
    if (/sk-[a-zA-Z0-9_-]{20,}/.test(text)) {
      warnings.push(`Step ${idx + 1}: OpenAI / Anthropic API Key`);
    }
    if (/AKIA[0-9A-Z]{16}/.test(text)) {
      warnings.push(`Step ${idx + 1}: AWS Access Key ID`);
    }
    if (text.includes('-----BEGIN') && text.includes('PRIVATE KEY-----')) {
      warnings.push(`Step ${idx + 1}: Private Key Block`);
    }
    return warnings;
  });

  const handleCanvasClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!activeFrame || !canvasRef.current) return;
    const rect = canvasRef.current.getBoundingClientRect();
    const x = Math.round(e.clientX - rect.left);
    const y = Math.round(e.clientY - rect.top);

    if (activeTool === 'callout') {
      const calloutCount = activeFrame.annotations.filter(
        (a) => a.kind.type === 'CalloutBadge'
      ).length + 1;

      const newAnno: Annotation = {
        id: `anno_${Date.now()}`,
        kind: { type: 'CalloutBadge', data: { number: calloutCount } },
        rect: { x: Math.max(0, x - 18), y: Math.max(0, y - 18), width: 36, height: 36 },
        color_hex: '#F97316',
      };

      setProject((prev) => ({
        ...prev,
        frames: prev.frames.map((f) =>
          f.id === activeFrame.id ? { ...f, annotations: [...f.annotations, newAnno] } : f
        ),
      }));
    } else if (activeTool === 'rectangle') {
      const newAnno: Annotation = {
        id: `anno_${Date.now()}`,
        kind: { type: 'Rectangle' },
        rect: { x: Math.max(0, x - 50), y: Math.max(0, y - 30), width: 100, height: 60 },
        color_hex: '#22C55E',
      };

      setProject((prev) => ({
        ...prev,
        frames: prev.frames.map((f) =>
          f.id === activeFrame.id ? { ...f, annotations: [...f.annotations, newAnno] } : f
        ),
      }));
    } else if (activeTool === 'redact') {
      const newRedact: Redaction = {
        id: `redact_${Date.now()}`,
        rect: { x: Math.max(0, x - 60), y: Math.max(0, y - 15), width: 120, height: 30 },
        style: { style: 'Solid', params: { color_hex: '#000000' } },
      };

      setProject((prev) => ({
        ...prev,
        frames: prev.frames.map((f) =>
          f.id === activeFrame.id ? { ...f, redactions: [...f.redactions, newRedact] } : f
        ),
      }));
    }
  };

  const handleExport = (format: 'markdown' | 'html' | 'both') => {
    setExportSuccess(`Exported project to ${format.toUpperCase()} successfully.`);
    setTimeout(() => {
      setShowExportModal(false);
      setExportSuccess(null);
    }, 1500);
  };

  return (
    <>
      {/* Header */}
      <header className="app-header">
        <div className="brand-section">
          <img src="/assets/logo.svg" alt="Dhap Logo" className="brand-logo" />
          <input
            type="text"
            className="project-title-input"
            value={project.title}
            onChange={(e) => setProject({ ...project, title: e.target.value })}
            title="Rename project"
          />
        </div>

        <div className="layout-switcher">
          <button
            className={`layout-btn ${layout === 'guide' ? 'active' : ''}`}
            onClick={() => setLayout('guide')}
          >
            Guide Editor
          </button>
          <button
            className={`layout-btn ${layout === 'timeline' ? 'active' : ''}`}
            onClick={() => setLayout('timeline')}
          >
            Timeline Mode
          </button>
          <button
            className={`layout-btn ${layout === 'preview' ? 'active' : ''}`}
            onClick={() => setLayout('preview')}
          >
            Live Preview
          </button>
        </div>

        <div className="header-actions">
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleOpenProject}
            accept=".json,.dhap"
            style={{ display: 'none' }}
          />
          <button className="btn btn-primary" style={{ background: '#ef4444', borderColor: '#ef4444', display: 'flex', alignItems: 'center' }} onClick={startLiveRecording}>
            <span className="record-pulse" />
            {isRecording ? 'Recording...' : 'Record Task'}
          </button>
          <button className="btn btn-secondary" onClick={() => fileInputRef.current?.click()}>
            Open
          </button>
          <button className="btn btn-secondary" onClick={handleSaveProject}>
            Save
          </button>
          <button className="btn btn-secondary" onClick={toggleTheme}>
            {theme === 'dark' ? '☀️ Light' : '🌙 Dark'}
          </button>
          <button className="btn btn-primary" onClick={() => setShowExportModal(true)}>
            Export Guide
          </button>
        </div>
      </header>

      {/* Main Workspace */}
      <div className="workspace">

        {/* Sidebar Steps */}
        <aside className="sidebar">
          <div className="sidebar-header">
            <h2>Workflow Steps ({project.steps.length})</h2>
            <button className="btn btn-secondary" style={{ padding: '0.25rem 0.5rem' }} onClick={handleAddStep}>
              + Add
            </button>
          </div>
          <div className="step-list">
            {project.steps.map((s, idx) => (
              <div
                key={s.id}
                className={`step-item ${s.id === activeStep.id ? 'active' : ''}`}
                onClick={() => setActiveStepId(s.id)}
              >
                <div className="step-item-head" style={{ justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', overflow: 'hidden' }}>
                    <span className="step-badge">Step {idx + 1}</span>
                    <span className="step-title">{s.title}</span>
                  </div>
                  <div style={{ display: 'flex', gap: '2px' }}>
                    <button
                      className="btn btn-secondary"
                      style={{ padding: '0 4px', fontSize: '11px', lineHeight: '18px', height: '20px' }}
                      title="Move Step Up"
                      onClick={(e) => handleMoveStepUp(s.id, e)}
                      disabled={idx === 0}
                    >
                      ↑
                    </button>
                    <button
                      className="btn btn-secondary"
                      style={{ padding: '0 4px', fontSize: '11px', lineHeight: '18px', height: '20px' }}
                      title="Move Step Down"
                      onClick={(e) => handleMoveStepDown(s.id, e)}
                      disabled={idx === project.steps.length - 1}
                    >
                      ↓
                    </button>
                  </div>
                </div>
                <p className="step-desc-preview">{s.instruction}</p>
                {s.source_timestamp_ms !== undefined && (
                  <span className="timestamp-pill">
                    {(s.source_timestamp_ms / 1000).toFixed(1)}s
                  </span>
                )}
              </div>
            ))}
          </div>
        </aside>

        {/* Center Panel */}
        {layout === 'preview' ? (
          <main className="editor-panel">
            <div className="editor-card" style={{ maxWidth: '800px' }}>
              <h1 style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>{project.title}</h1>
              <p style={{ color: 'var(--text-muted)', marginBottom: '2rem' }}>
                {project.description}
              </p>
              {project.steps.map((step, idx) => (
                <div
                  key={step.id}
                  style={{
                    background: 'var(--bg-panel)',
                    padding: '1.25rem',
                    borderRadius: '12px',
                    marginBottom: '1.5rem',
                    border: '1px solid var(--border-subtle)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.75rem' }}>
                    <span className="step-badge">Step {idx + 1}</span>
                    <h3 style={{ fontSize: '1.2rem' }}>{step.title}</h3>
                  </div>
                  <p style={{ marginBottom: '1rem' }}>{step.instruction}</p>
                  {step.expected_result && (
                    <div style={{ padding: '0.65rem 0.85rem', background: 'rgba(34, 197, 94, 0.1)', borderLeft: '4px solid #22c55e', borderRadius: '4px', marginBottom: '0.75rem' }}>
                      <strong>Expected Result:</strong> {step.expected_result}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </main>
        ) : (
          <main className="editor-panel">
            <div className="editor-card">
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <span className="step-badge">Step {activeStep.order}</span>
                  <button
                    className="btn btn-secondary"
                    style={{ padding: '0.2rem 0.5rem', fontSize: '0.85rem' }}
                    onClick={(e) => handleMoveStepUp(activeStep.id, e)}
                    disabled={activeStep.order <= 1}
                  >
                    ↑ Move Up
                  </button>
                  <button
                    className="btn btn-secondary"
                    style={{ padding: '0.2rem 0.5rem', fontSize: '0.85rem' }}
                    onClick={(e) => handleMoveStepDown(activeStep.id, e)}
                    disabled={activeStep.order >= project.steps.length}
                  >
                    ↓ Move Down
                  </button>
                </div>
                <button
                  className="btn btn-secondary"
                  style={{ color: '#ef4444', borderColor: '#ef4444' }}
                  onClick={() => handleDeleteStep(activeStep.id)}
                >
                  Delete Step
                </button>
              </div>

              <div className="form-group">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.35rem' }}>
                  <label style={{ margin: 0 }}>Step Title</label>
                  <button
                    className="btn btn-secondary"
                    style={{ fontSize: '0.8rem', padding: '0.2rem 0.5rem' }}
                    onClick={handleAiAssist}
                    disabled={aiGenerating}
                  >
                    {aiGenerating ? '⏳ Scanning Privacy...' : '✨ AI Assist'}
                  </button>
                </div>
                {aiStatus && (
                  <div style={{ fontSize: '0.8rem', color: '#38bdf8', marginBottom: '0.4rem' }}>
                    {aiStatus}
                  </div>
                )}
                <input
                  type="text"
                  className="form-control"
                  value={activeStep.title}
                  onChange={(e) => handleUpdateStep({ title: e.target.value })}
                />
              </div>

              <div className="form-group">
                <label>Instruction</label>
                <textarea
                  className="form-control"
                  value={activeStep.instruction}
                  onChange={(e) => handleUpdateStep({ instruction: e.target.value })}
                />
              </div>

              <div className="form-group">
                <label>Expected Result (Optional)</label>
                <input
                  type="text"
                  className="form-control"
                  value={activeStep.expected_result || ''}
                  onChange={(e) => handleUpdateStep({ expected_result: e.target.value })}
                  placeholder="e.g. Cluster status shows healthy green badge"
                />
              </div>

              {/* Screenshot Canvas & Annotation Controls */}
              {activeFrame && (
                <div className="form-group">
                  <label>Screenshot & Annotations (Click on image to apply)</label>
                  <div className="canvas-toolbar">
                    <button
                      className={`tool-btn ${activeTool === 'select' ? 'active' : ''}`}
                      onClick={() => setActiveTool('select')}
                    >
                      ✋ Select
                    </button>
                    <button
                      className={`tool-btn ${activeTool === 'callout' ? 'active' : ''}`}
                      onClick={() => setActiveTool('callout')}
                    >
                      🔢 Number Badge
                    </button>
                    <button
                      className={`tool-btn ${activeTool === 'rectangle' ? 'active' : ''}`}
                      onClick={() => setActiveTool('rectangle')}
                    >
                      🟩 Highlight Box
                    </button>
                    <button
                      className={`tool-btn ${activeTool === 'redact' ? 'active' : ''}`}
                      onClick={() => setActiveTool('redact')}
                      style={{ color: '#ef4444' }}
                    >
                      ⬛ Solid Redaction
                    </button>
                  </div>

                  <div
                    ref={canvasRef}
                    className="canvas-container"
                    onClick={handleCanvasClick}
                    style={{ position: 'relative', width: '100%', height: '380px', cursor: activeTool === 'select' ? 'default' : 'crosshair' }}
                  >
                    {/* Simulated terminal preview background */}
                    <div
                      style={{
                        width: '100%',
                        height: '100%',
                        background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'center',
                        alignItems: 'center',
                        color: '#64748b',
                        fontFamily: 'monospace',
                        fontSize: '1.1rem',
                        position: 'absolute',
                      }}
                    >
                      <span>[Screen Recording Frame @ {activeFrame.timestamp_ms}ms]</span>
                      <span style={{ fontSize: '0.85rem', color: '#475569', marginTop: '0.5rem' }}>
                        Active Tool: {activeTool.toUpperCase()} (Click to place)
                      </span>
                    </div>

                    {/* Rendered Annotations */}
                    {activeFrame.annotations.map((anno) => (
                      <div
                        key={anno.id}
                        style={{
                          position: 'absolute',
                          left: `${anno.rect.x}px`,
                          top: `${anno.rect.y}px`,
                          width: `${anno.rect.width}px`,
                          height: `${anno.rect.height}px`,
                          border: anno.kind.type === 'Rectangle' ? `3px solid ${anno.color_hex}` : 'none',
                          backgroundColor:
                            anno.kind.type === 'CalloutBadge' ? anno.color_hex : 'rgba(34, 197, 94, 0.2)',
                          borderRadius: anno.kind.type === 'CalloutBadge' ? '50%' : '6px',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          color: '#fff',
                          fontWeight: 700,
                          fontSize: '0.85rem',
                          pointerEvents: 'none',
                        }}
                      >
                        {anno.kind.type === 'CalloutBadge' && anno.kind.data.number}
                      </div>
                    ))}

                    {/* Rendered Redactions */}
                    {activeFrame.redactions.map((redact) => (
                      <div
                        key={redact.id}
                        style={{
                          position: 'absolute',
                          left: `${redact.rect.x}px`,
                          top: `${redact.rect.y}px`,
                          width: `${redact.rect.width}px`,
                          height: `${redact.rect.height}px`,
                          backgroundColor: '#000000',
                          border: '1px dashed #ef4444',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          color: '#ef4444',
                          fontSize: '0.75rem',
                          fontWeight: 700,
                          pointerEvents: 'none',
                        }}
                      >
                        REDACTED
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </main>
        )}
      </div>

      {/* Export Modal */}
      {showExportModal && (
        <div className="modal-overlay" onClick={() => setShowExportModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>Export Guide Documentation</h3>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem' }}>
              Generate reproducible, offline documentation bundles from this project.
            </p>

            <div className="form-group" style={{ marginBottom: '1rem' }}>
              <label>Export Template Style</label>
              <select
                className="form-control"
                value={selectedTemplate}
                onChange={(e) => setSelectedTemplate(e.target.value as any)}
              >
                <option value="standard">Standard Guide (Interactive Step-by-Step)</option>
                <option value="sop">SOP / Compliance (Document ID & Sign-off Block)</option>
                <option value="classroom">Classroom Handout (Exercise Checklist & Quiz)</option>
                <option value="github">Open Source GitHub README</option>
              </select>
            </div>

            <div style={{ marginBottom: '1.25rem' }}>
              <div style={{ fontSize: '0.85rem', fontWeight: 600, marginBottom: '0.35rem' }}>
                Privacy & Redaction Preflight
              </div>
              {detectedSecrets.length > 0 ? (
                <div style={{ background: 'rgba(239, 68, 68, 0.1)', border: '1px solid #ef4444', borderRadius: '6px', padding: '0.65rem 0.85rem' }}>
                  <div style={{ color: '#ef4444', fontWeight: 600, fontSize: '0.85rem' }}>
                    ⚠️ {detectedSecrets.length} Potential Unredacted Credential(s) Detected!
                  </div>
                  <ul style={{ fontSize: '0.8rem', color: '#fca5a5', marginTop: '0.35rem', paddingLeft: '1.25rem' }}>
                    {detectedSecrets.map((sec, idx) => (
                      <li key={idx}>{sec}</li>
                    ))}
                  </ul>
                </div>
              ) : (
                <div style={{ background: 'rgba(34, 197, 94, 0.1)', border: '1px solid #22c55e', borderRadius: '6px', padding: '0.6rem 0.85rem', color: '#22c55e', fontSize: '0.85rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <span>✓</span>
                  <span><strong>Clean Preflight:</strong> Zero credentials or API keys detected.</span>
                </div>
              )}
            </div>

            <div className="export-options">
              <label className="export-option-label">
                <input type="radio" name="format" defaultChecked />
                <div>
                  <strong>Markdown + Local Assets</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    README.md with relative image links
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input type="radio" name="format" />
                <div>
                  <strong>Standalone Responsive HTML Guide</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Interactive stepper with dark/light mode and lightbox
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input type="checkbox" defaultChecked />
                <div>
                  <strong>Bake Irreversible Redactions</strong>
                  <div style={{ fontSize: '0.8rem', color: '#22c55e' }}>
                    Permanently overwrites sensitive pixels in memory before export
                  </div>
                </div>
              </label>
            </div>

            {exportSuccess ? (
              <div style={{ padding: '0.75rem', background: 'rgba(34, 197, 94, 0.1)', color: '#22c55e', borderRadius: '6px', textAlign: 'center', fontWeight: 600 }}>
                {exportSuccess}
              </div>
            ) : (
              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '1.5rem' }}>
                <button className="btn btn-secondary" onClick={() => setShowExportModal(false)}>
                  Cancel
                </button>
                <button className="btn btn-primary" onClick={() => handleExport('both')}>
                  Start Export
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Workflow Recording Modal */}
      {showRecordModal && (
        <div className="modal-overlay">
          <div className="modal-content" style={{ maxWidth: '560px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center' }}>
                <span className="record-pulse" />
                <h3 style={{ margin: 0 }}>Active Workflow Recording</h3>
              </div>
              <span className="step-badge" style={{ background: '#ef4444' }}>LIVE</span>
            </div>

            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.5rem' }}>
              Perform actions on your screen. Tap "Mark Step" whenever you complete an action or state change.
            </p>

            <div className="recorder-timer-display">
              {Math.floor(recordingSec / 60).toString().padStart(2, '0')}:
              {(recordingSec % 60).toString().padStart(2, '0')}
            </div>

            <div style={{ display: 'flex', gap: '0.75rem', marginBottom: '1.25rem' }}>
              <button
                className="btn btn-primary"
                style={{ flex: 1, padding: '0.85rem', fontSize: '1rem', background: '#2563eb', borderColor: '#2563eb' }}
                onClick={markRecordingStep}
              >
                📍 Mark Step ({markers.length} captured)
              </button>
            </div>

            <div className="marker-feed">
              {markers.length === 0 ? (
                <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.85rem', padding: '1rem 0' }}>
                  No markers captured yet. Click "Mark Step" or press spacebar as you work.
                </div>
              ) : (
                markers.map((m, idx) => (
                  <div key={idx} className="marker-feed-item">
                    <span>
                      <strong style={{ color: 'var(--primary)', marginRight: '0.5rem' }}>#{idx + 1}</strong>
                      {m.label}
                    </span>
                    <span className="timestamp-pill">
                      {(m.timestampMs / 1000).toFixed(1)}s
                    </span>
                  </div>
                ))
              )}
            </div>

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.75rem' }}>
              <button
                className="btn btn-secondary"
                onClick={() => {
                  if (recordingTimerRef.current) clearInterval(recordingTimerRef.current);
                  setIsRecording(false);
                  setShowRecordModal(false);
                }}
              >
                Cancel
              </button>
              <button
                className="btn btn-primary"
                style={{ background: '#22c55e', borderColor: '#22c55e' }}
                onClick={stopAndFinalizeRecording}
              >
                Done & Generate Guide
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

