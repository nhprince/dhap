import React, { useState, useRef, useEffect } from 'react';
import {
  Project,
  GuideStep,
  Annotation,
  Redaction,
  LayoutMode,
  ActionVerb,
  ChecklistItem,
  ProjectSnapshot,
} from './types';

// Sample initial project showcasing Dhap's beast-tier capabilities
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
      action_kind: 'command',
      code_snippet: {
        language: 'bash',
        code: 'dhap auth login --endpoint https://api.dhap.internal\ndhap cluster status --output json',
      },
      checklist: [
        { id: 'chk_1', text: 'Verify corporate VPN is active', completed: true },
        { id: 'chk_2', text: 'Check cluster ingress latency < 50ms', completed: false },
      ],
      tags: ['deployment', 'cli', 'auth'],
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
      action_kind: 'verify',
      checklist: [
        { id: 'chk_3', text: 'All 3 ingress pods report healthy', completed: true },
      ],
      tags: ['monitoring', 'healthcheck'],
    },
  ],
  export_settings: {
    default_format: 'markdown,html,json,gif',
    bake_redactions: true,
    include_timestamps: true,
    output_dir: 'dist_docs',
  },
  snapshots: [
    {
      id: 'snap_v1',
      label: 'v1.0 Baseline',
      created_at: new Date(Date.now() - 3600000).toISOString(),
      step_count: 2,
      titles: ['Authenticate to Gateway', 'Verify Cluster Healthcheck'],
    },
  ],
};

const ACTION_VERBS: ActionVerb[] = [
  'click',
  'type',
  'command',
  'verify',
  'select',
  'navigate',
  'shortcut',
  'wait',
];

export default function App() {
  const [project, setProject] = useState<Project>(initialProject);
  const [activeStepId, setActiveStepId] = useState<string>(project.steps[0].id);
  const [layout, setLayout] = useState<LayoutMode>('guide');
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [activeTool, setActiveTool] = useState<'select' | 'callout' | 'rectangle' | 'redact'>('select');
  const [showExportModal, setShowExportModal] = useState<boolean>(false);
  const [selectedTemplate, setSelectedTemplate] = useState<'standard' | 'sop' | 'classroom' | 'github'>('standard');
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);
  const [selectedExportFormat, setSelectedExportFormat] = useState<'html' | 'markdown' | 'json' | 'gif' | 'all'>('html');

  // Search & Filters
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [actionFilter, setActionFilter] = useState<ActionVerb | 'all'>('all');

  // Stats & Snapshot Drawer
  const [showStatsModal, setShowStatsModal] = useState<boolean>(false);
  const [newSnapshotLabel, setNewSnapshotLabel] = useState<string>('');

  // Speech Narration State
  const [isNarrating, setIsNarrating] = useState<boolean>(false);
  const [copiedCodeId, setCopiedCodeId] = useState<string | null>(null);

  // New item inputs
  const [newChecklistText, setNewChecklistText] = useState<string>('');
  const [newTagText, setNewTagText] = useState<string>('');

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

  // Filtered steps
  const filteredSteps = project.steps.filter((s) => {
    const matchesFilter = actionFilter === 'all' || s.action_kind === actionFilter;
    if (!matchesFilter) return false;
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      s.title.toLowerCase().includes(q) ||
      s.instruction.toLowerCase().includes(q) ||
      (s.expected_result && s.expected_result.toLowerCase().includes(q)) ||
      (s.notes && s.notes.toLowerCase().includes(q)) ||
      (s.tags && s.tags.some((t) => t.toLowerCase().includes(q))) ||
      (s.code_snippet && s.code_snippet.code.toLowerCase().includes(q))
    );
  });

  const activeStep = project.steps.find((s) => s.id === activeStepId) || project.steps[0];
  const activeFrame = activeStep?.frame_ids?.length > 0
    ? project.frames.find((f) => f.id === activeStep.frame_ids[0])
    : null;

  const currentStepIndex = project.steps.findIndex((s) => s.id === activeStep.id);

  // Keyboard navigation for presentation mode
  useEffect(() => {
    if (layout !== 'presentation') return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' || e.key === ' ') {
        e.preventDefault();
        if (currentStepIndex < project.steps.length - 1) {
          setActiveStepId(project.steps[currentStepIndex + 1].id);
        }
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        if (currentStepIndex > 0) {
          setActiveStepId(project.steps[currentStepIndex - 1].id);
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        setLayout('guide');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [layout, currentStepIndex, project.steps]);

  // Audio Speech Narration
  const toggleSpeechNarration = () => {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
      alert('Web Speech API is not supported in this browser.');
      return;
    }

    if (isNarrating) {
      window.speechSynthesis.cancel();
      setIsNarrating(false);
      return;
    }

    window.speechSynthesis.cancel();
    const narrationText = `Step ${activeStep.order}: ${activeStep.title}. ${activeStep.instruction}. ${
      activeStep.expected_result ? 'Expected outcome: ' + activeStep.expected_result : ''
    }`;
    const utterance = new SpeechSynthesisUtterance(narrationText);
    utterance.rate = 1.0;
    utterance.pitch = 1.0;
    utterance.onend = () => setIsNarrating(false);
    utterance.onerror = () => setIsNarrating(false);

    window.speechSynthesis.speak(utterance);
    setIsNarrating(true);
  };

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCodeId(id);
    setTimeout(() => setCopiedCodeId(null), 1500);
  };

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
      setAiStatus('Extracting action verbs & expected outcome...');
      setTimeout(() => {
        const refinedTitle = activeStep.title.replace(/^Step \d+:\s*/, '');
        handleUpdateStep({
          title: `Configure ${refinedTitle || 'Component'}`,
          action_kind: 'command',
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
      action_kind: 'click',
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

  // Checklist Helpers
  const handleToggleChecklist = (checkId: string) => {
    const list = activeStep.checklist || [];
    const updated = list.map((item) =>
      item.id === checkId ? { ...item, completed: !item.completed } : item
    );
    handleUpdateStep({ checklist: updated });
  };

  const handleAddChecklistItem = () => {
    if (!newChecklistText.trim()) return;
    const list = activeStep.checklist || [];
    const newItem: ChecklistItem = {
      id: `chk_${Date.now()}`,
      text: newChecklistText.trim(),
      completed: false,
    };
    handleUpdateStep({ checklist: [...list, newItem] });
    setNewChecklistText('');
  };

  const handleDeleteChecklistItem = (checkId: string) => {
    const list = activeStep.checklist || [];
    handleUpdateStep({ checklist: list.filter((i) => i.id !== checkId) });
  };

  // Tag Helpers
  const handleAddTag = () => {
    if (!newTagText.trim()) return;
    const tags = activeStep.tags || [];
    if (!tags.includes(newTagText.trim())) {
      handleUpdateStep({ tags: [...tags, newTagText.trim()] });
    }
    setNewTagText('');
  };

  const handleRemoveTag = (tag: string) => {
    const tags = activeStep.tags || [];
    handleUpdateStep({ tags: tags.filter((t) => t !== tag) });
  };

  // Snapshot Creation
  const handleCreateSnapshot = () => {
    const label = newSnapshotLabel.trim() || `Snapshot #${(project.snapshots?.length || 0) + 1}`;
    const newSnapshot: ProjectSnapshot = {
      id: `snap_${Date.now()}`,
      label,
      created_at: new Date().toISOString(),
      step_count: project.steps.length,
      titles: project.steps.map((s) => s.title),
    };
    setProject((prev) => ({
      ...prev,
      snapshots: [newSnapshot, ...(prev.snapshots || [])],
    }));
    setNewSnapshotLabel('');
  };

  const detectedSecrets = project.steps.flatMap((step, idx) => {
    const text = `${step.title} ${step.instruction} ${step.expected_result || ''} ${step.notes || ''} ${
      step.code_snippet?.code || ''
    }`;
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

  const handleExport = (format: 'html' | 'markdown' | 'json' | 'gif' | 'all') => {
    if (format === 'json') {
      const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(project, null, 2));
      const a = document.createElement('a');
      a.href = dataStr;
      a.download = `${project.title.toLowerCase().replace(/[^a-z0-9]/g, '_')}_guide.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
    } else if (format === 'markdown') {
      let md = `# ${project.title}\n\n${project.description || ''}\n\n`;
      project.steps.forEach((s, idx) => {
        md += `## Step ${idx + 1}: ${s.title}\n\n`;
        if (s.action_kind) md += `**Action:** \`${s.action_kind.toUpperCase()}\`\n\n`;
        md += `${s.instruction}\n\n`;
        if (s.code_snippet) {
          md += `\`\`\`${s.code_snippet.language}\n${s.code_snippet.code}\n\`\`\`\n\n`;
        }
        if (s.expected_result) md += `> **Expected Result:** ${s.expected_result}\n\n`;
        if (s.checklist && s.checklist.length > 0) {
          md += `### Checklist\n`;
          s.checklist.forEach((c) => {
            md += `- [${c.completed ? 'x' : ' '}] ${c.text}\n`;
          });
          md += `\n`;
        }
      });
      const dataStr = 'data:text/markdown;charset=utf-8,' + encodeURIComponent(md);
      const a = document.createElement('a');
      a.href = dataStr;
      a.download = 'README.md';
      document.body.appendChild(a);
      a.click();
      a.remove();
    }

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
            className={`layout-btn ${layout === 'preview' ? 'active' : ''}`}
            onClick={() => setLayout('preview')}
          >
            Live Preview
          </button>
          <button
            className={`layout-btn ${layout === 'presentation' ? 'active' : ''}`}
            onClick={() => setLayout('presentation')}
          >
            📽️ Slide Deck
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
          <button
            className="btn btn-primary"
            style={{ background: '#ef4444', borderColor: '#ef4444', display: 'flex', alignItems: 'center' }}
            onClick={startLiveRecording}
          >
            <span className="record-pulse" />
            {isRecording ? 'Recording...' : 'Record Task'}
          </button>
          <button className="btn btn-secondary" onClick={() => setShowStatsModal(true)}>
            📊 Stats & Diff
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

      {/* Main Workspace Layout Switcher */}
      {layout === 'presentation' ? (
        /* Presentation Slide Mode */
        <div className="presentation-container">
          <div className="presentation-progress-bar">
            <div
              className="presentation-progress-fill"
              style={{
                width: `${((currentStepIndex + 1) / project.steps.length) * 100}%`,
              }}
            />
          </div>

          <div className="presentation-header">
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
              <button
                className="btn btn-secondary"
                onClick={() => setLayout('guide')}
                style={{ fontSize: '0.8rem' }}
              >
                ✕ Exit Deck
              </button>
              <h2 style={{ fontSize: '1.1rem', fontWeight: 700, margin: 0 }}>
                {project.title}
              </h2>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <button
                className={`btn ${isNarrating ? 'btn-primary' : 'btn-secondary'}`}
                onClick={toggleSpeechNarration}
                style={{ fontSize: '0.85rem' }}
              >
                {isNarrating ? '⏹ Stop Speech' : '🔊 Read Aloud'}
              </button>
              <span className="step-badge" style={{ fontSize: '0.85rem', padding: '0.25rem 0.75rem' }}>
                Slide {currentStepIndex + 1} of {project.steps.length}
              </span>
            </div>
          </div>

          <div className="presentation-slide-card">
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '1.25rem' }}>
              <span className="step-badge" style={{ fontSize: '0.9rem', padding: '0.3rem 0.8rem' }}>
                Step {activeStep.order}
              </span>
              {activeStep.action_kind && (
                <span className={`action-badge ${activeStep.action_kind}`}>
                  {activeStep.action_kind}
                </span>
              )}
              <h1 style={{ fontSize: '1.85rem', fontWeight: 800, margin: 0 }}>
                {activeStep.title}
              </h1>
            </div>

            <p style={{ fontSize: '1.2rem', lineHeight: 1.6, marginBottom: '1.5rem', color: 'var(--text-main)' }}>
              {activeStep.instruction}
            </p>

            {/* Expected Result Callout */}
            {activeStep.expected_result && (
              <div
                style={{
                  background: 'rgba(34, 197, 94, 0.1)',
                  borderLeft: '5px solid #22c55e',
                  padding: '1rem 1.25rem',
                  borderRadius: '8px',
                  marginBottom: '1.5rem',
                  fontSize: '1rem',
                }}
              >
                <strong style={{ color: '#22c55e' }}>✓ Expected Result:</strong>{' '}
                {activeStep.expected_result}
              </div>
            )}

            {/* Code Snippet in Presentation */}
            {activeStep.code_snippet && activeStep.code_snippet.code && (
              <div className="code-block-container" style={{ marginBottom: '1.5rem' }}>
                <div className="code-block-header">
                  <span className="code-lang-tag">{activeStep.code_snippet.language}</span>
                  <button
                    className="code-copy-btn"
                    onClick={() => copyToClipboard(activeStep.code_snippet!.code, 'slide_code')}
                  >
                    {copiedCodeId === 'slide_code' ? '✓ Copied' : 'Copy'}
                  </button>
                </div>
                <pre className="code-block-pre">{activeStep.code_snippet.code}</pre>
              </div>
            )}

            {/* Checklist in Presentation */}
            {activeStep.checklist && activeStep.checklist.length > 0 && (
              <div style={{ marginBottom: '1.5rem' }}>
                <h4 style={{ fontSize: '0.9rem', color: 'var(--text-muted)', marginBottom: '0.5rem', textTransform: 'uppercase' }}>
                  Verification Checklist
                </h4>
                <div className="checklist-container">
                  {activeStep.checklist.map((c) => (
                    <label key={c.id} className={`checklist-item ${c.completed ? 'done' : ''}`} style={{ cursor: 'pointer' }}>
                      <input
                        type="checkbox"
                        checked={c.completed || false}
                        onChange={() => handleToggleChecklist(c.id)}
                      />
                      <span>{c.text}</span>
                    </label>
                  ))}
                </div>
              </div>
            )}

            {/* Frame / Screenshot Preview */}
            {activeFrame && (
              <div
                style={{
                  background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)',
                  borderRadius: '12px',
                  border: '1px solid var(--border-subtle)',
                  padding: '2.5rem',
                  textAlign: 'center',
                  color: '#94a3b8',
                  fontFamily: 'monospace',
                  fontSize: '0.95rem',
                  marginTop: 'auto',
                }}
              >
                [Screenshot Frame captured at {activeFrame.timestamp_ms}ms with {activeFrame.annotations.length} annotation(s)]
              </div>
            )}
          </div>

          <div className="presentation-controls">
            <button
              className="btn btn-secondary"
              onClick={() => currentStepIndex > 0 && setActiveStepId(project.steps[currentStepIndex - 1].id)}
              disabled={currentStepIndex === 0}
              style={{ padding: '0.6rem 1.25rem' }}
            >
              ← Previous Slide
            </button>
            <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Press <kbd style={{ background: 'var(--bg-panel)', padding: '2px 6px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>←</kbd> / <kbd style={{ background: 'var(--bg-panel)', padding: '2px 6px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>→</kbd> or <kbd style={{ background: 'var(--bg-panel)', padding: '2px 6px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>Space</kbd> to navigate
            </div>
            <button
              className="btn btn-primary"
              onClick={() => currentStepIndex < project.steps.length - 1 && setActiveStepId(project.steps[currentStepIndex + 1].id)}
              disabled={currentStepIndex === project.steps.length - 1}
              style={{ padding: '0.6rem 1.25rem' }}
            >
              Next Slide →
            </button>
          </div>
        </div>
      ) : (
        /* Standard Workspace (Guide Editor & Live Preview) */
        <div className="workspace">
          {/* Sidebar Steps */}
          <aside className="sidebar">
            <div className="sidebar-header">
              <h2>Steps ({filteredSteps.length} of {project.steps.length})</h2>
              <button className="btn btn-secondary" style={{ padding: '0.25rem 0.5rem' }} onClick={handleAddStep}>
                + Add
              </button>
            </div>

            {/* Sidebar Search and Action Filters */}
            <div className="sidebar-search-container">
              <input
                type="text"
                className="sidebar-search-input"
                placeholder="🔍 Search title, code, tags..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              <div className="action-filter-chips">
                <span
                  className={`action-chip ${actionFilter === 'all' ? 'active' : ''}`}
                  onClick={() => setActionFilter('all')}
                >
                  All
                </span>
                {ACTION_VERBS.map((v) => (
                  <span
                    key={v}
                    className={`action-chip ${actionFilter === v ? 'active' : ''}`}
                    onClick={() => setActionFilter(v)}
                  >
                    {v}
                  </span>
                ))}
              </div>
            </div>

            <div className="step-list">
              {filteredSteps.map((s, idx) => (
                <div
                  key={s.id}
                  className={`step-item ${s.id === activeStep.id ? 'active' : ''}`}
                  onClick={() => setActiveStepId(s.id)}
                >
                  <div className="step-item-head" style={{ justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', overflow: 'hidden' }}>
                      <span className="step-badge">Step {s.order}</span>
                      {s.action_kind && (
                        <span className={`action-badge ${s.action_kind}`} style={{ fontSize: '0.65rem', padding: '0.1rem 0.35rem' }}>
                          {s.action_kind}
                        </span>
                      )}
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
                  <div style={{ display: 'flex', gap: '0.35rem', alignItems: 'center', flexWrap: 'wrap', marginTop: '0.35rem' }}>
                    {s.source_timestamp_ms !== undefined && (
                      <span className="timestamp-pill">
                        {(s.source_timestamp_ms / 1000).toFixed(1)}s
                      </span>
                    )}
                    {s.code_snippet && (
                      <span className="timestamp-pill" style={{ color: '#a855f7', background: 'rgba(168, 85, 247, 0.1)' }}>
                        code: {s.code_snippet.language}
                      </span>
                    )}
                    {s.checklist && s.checklist.length > 0 && (
                      <span className="timestamp-pill" style={{ color: '#22c55e', background: 'rgba(34, 197, 94, 0.1)' }}>
                        ✓ {s.checklist.filter((c) => c.completed).length}/{s.checklist.length}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </aside>

          {/* Center Panel */}
          {layout === 'preview' ? (
            <main className="editor-panel">
              <div className="editor-card" style={{ maxWidth: '820px' }}>
                <h1 style={{ fontSize: '2.2rem', marginBottom: '0.5rem', fontWeight: 800 }}>{project.title}</h1>
                <p style={{ color: 'var(--text-muted)', fontSize: '1.05rem', marginBottom: '2rem' }}>
                  {project.description}
                </p>
                {project.steps.map((step, idx) => (
                  <div
                    key={step.id}
                    style={{
                      background: 'var(--bg-panel)',
                      padding: '1.5rem',
                      borderRadius: '14px',
                      marginBottom: '1.75rem',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.6rem', marginBottom: '0.75rem' }}>
                      <span className="step-badge">Step {idx + 1}</span>
                      {step.action_kind && (
                        <span className={`action-badge ${step.action_kind}`}>
                          {step.action_kind}
                        </span>
                      )}
                      <h3 style={{ fontSize: '1.25rem', fontWeight: 700 }}>{step.title}</h3>
                    </div>

                    <p style={{ marginBottom: '1rem', lineHeight: 1.5 }}>{step.instruction}</p>

                    {/* Expected Result */}
                    {step.expected_result && (
                      <div
                        style={{
                          padding: '0.75rem 1rem',
                          background: 'rgba(34, 197, 94, 0.1)',
                          borderLeft: '4px solid #22c55e',
                          borderRadius: '4px',
                          marginBottom: '0.85rem',
                        }}
                      >
                        <strong style={{ color: '#22c55e' }}>Expected Result:</strong> {step.expected_result}
                      </div>
                    )}

                    {/* Code Snippet in Live Preview */}
                    {step.code_snippet && step.code_snippet.code && (
                      <div className="code-block-container" style={{ marginBottom: '0.85rem' }}>
                        <div className="code-block-header">
                          <span className="code-lang-tag">{step.code_snippet.language}</span>
                          <button
                            className="code-copy-btn"
                            onClick={() => copyToClipboard(step.code_snippet!.code, `prev_${step.id}`)}
                          >
                            {copiedCodeId === `prev_${step.id}` ? '✓ Copied' : 'Copy'}
                          </button>
                        </div>
                        <pre className="code-block-pre">{step.code_snippet.code}</pre>
                      </div>
                    )}

                    {/* Interactive Checklist in Live Preview */}
                    {step.checklist && step.checklist.length > 0 && (
                      <div className="checklist-container">
                        {step.checklist.map((c) => (
                          <label key={c.id} className={`checklist-item ${c.completed ? 'done' : ''}`}>
                            <input
                              type="checkbox"
                              checked={c.completed || false}
                              onChange={() => {
                                const updated = (step.checklist || []).map((item) =>
                                  item.id === c.id ? { ...item, completed: !item.completed } : item
                                );
                                setProject((prev) => ({
                                  ...prev,
                                  steps: prev.steps.map((st) => (st.id === step.id ? { ...st, checklist: updated } : st)),
                                }));
                              }}
                            />
                            <span>{c.text}</span>
                          </label>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </main>
          ) : (
            <main className="editor-panel">
              <div className="editor-card">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.25rem' }}>
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

                {/* Action Verb Selector */}
                <div className="form-group">
                  <label>Action Category (Verb)</label>
                  <div style={{ display: 'flex', gap: '0.4rem', flexWrap: 'wrap' }}>
                    {ACTION_VERBS.map((verb) => (
                      <button
                        key={verb}
                        type="button"
                        className={`action-badge ${verb}`}
                        style={{
                          cursor: 'pointer',
                          opacity: activeStep.action_kind === verb ? 1 : 0.45,
                          transform: activeStep.action_kind === verb ? 'scale(1.05)' : 'none',
                          border: activeStep.action_kind === verb ? '2px solid' : '1px solid transparent',
                        }}
                        onClick={() => handleUpdateStep({ action_kind: verb })}
                      >
                        {verb}
                      </button>
                    ))}
                  </div>
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

                {/* Code Snippet Editor */}
                <div className="form-group">
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.35rem' }}>
                    <label style={{ margin: 0 }}>Code / Terminal Command (Optional)</label>
                    <select
                      className="form-control"
                      style={{ width: '130px', padding: '0.2rem 0.5rem', fontSize: '0.8rem' }}
                      value={activeStep.code_snippet?.language || 'bash'}
                      onChange={(e) =>
                        handleUpdateStep({
                          code_snippet: {
                            language: e.target.value,
                            code: activeStep.code_snippet?.code || '',
                          },
                        })
                      }
                    >
                      <option value="bash">bash</option>
                      <option value="sh">sh</option>
                      <option value="rust">rust</option>
                      <option value="python">python</option>
                      <option value="typescript">typescript</option>
                      <option value="javascript">javascript</option>
                      <option value="json">json</option>
                      <option value="yaml">yaml</option>
                      <option value="dockerfile">dockerfile</option>
                    </select>
                  </div>
                  <textarea
                    className="form-control"
                    style={{ fontFamily: 'var(--font-mono)', minHeight: '80px', fontSize: '0.85rem' }}
                    placeholder="e.g. cargo build --release"
                    value={activeStep.code_snippet?.code || ''}
                    onChange={(e) =>
                      handleUpdateStep({
                        code_snippet: {
                          language: activeStep.code_snippet?.language || 'bash',
                          code: e.target.value,
                        },
                      })
                    }
                  />
                </div>

                {/* Interactive Checklist Editor */}
                <div className="form-group">
                  <label>Verification Checklist ({activeStep.checklist?.length || 0})</label>
                  <div className="checklist-container">
                    {(activeStep.checklist || []).map((c) => (
                      <div key={c.id} className="checklist-item">
                        <input
                          type="checkbox"
                          checked={c.completed || false}
                          onChange={() => handleToggleChecklist(c.id)}
                        />
                        <span style={{ flex: 1, textDecoration: c.completed ? 'line-through' : 'none' }}>
                          {c.text}
                        </span>
                        <button
                          type="button"
                          className="btn btn-secondary"
                          style={{ padding: '0 5px', fontSize: '11px', height: '20px' }}
                          onClick={() => handleDeleteChecklistItem(c.id)}
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                    <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.35rem' }}>
                      <input
                        type="text"
                        className="form-control"
                        placeholder="Add checklist sub-task..."
                        value={newChecklistText}
                        onChange={(e) => setNewChecklistText(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleAddChecklistItem()}
                        style={{ padding: '0.4rem 0.6rem', fontSize: '0.85rem' }}
                      />
                      <button className="btn btn-secondary" type="button" onClick={handleAddChecklistItem}>
                        Add
                      </button>
                    </div>
                  </div>
                </div>

                {/* Tags Editor */}
                <div className="form-group">
                  <label>Tags</label>
                  <div style={{ display: 'flex', gap: '0.4rem', flexWrap: 'wrap', alignItems: 'center' }}>
                    {(activeStep.tags || []).map((tag) => (
                      <span
                        key={tag}
                        className="action-chip"
                        style={{ background: 'var(--bg-hover)', color: 'var(--text-main)', display: 'inline-flex', alignItems: 'center', gap: '0.3rem' }}
                      >
                        #{tag}
                        <button
                          type="button"
                          onClick={() => handleRemoveTag(tag)}
                          style={{ background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer', fontWeight: 'bold' }}
                        >
                          ×
                        </button>
                      </span>
                    ))}
                    <input
                      type="text"
                      className="form-control"
                      placeholder="+ Add tag"
                      value={newTagText}
                      onChange={(e) => setNewTagText(e.target.value)}
                      onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
                      style={{ width: '110px', padding: '0.2rem 0.5rem', fontSize: '0.8rem' }}
                    />
                  </div>
                </div>

                {/* Screenshot Canvas & Annotation Controls */}
                {activeFrame && (
                  <div className="form-group">
                    <label>Screenshot & Annotations (Click on canvas to apply)</label>
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
                      style={{
                        position: 'relative',
                        width: '100%',
                        height: '380px',
                        cursor: activeTool === 'select' ? 'default' : 'crosshair',
                      }}
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
      )}

      {/* Project Statistics & Version History Modal */}
      {showStatsModal && (
        <div className="modal-overlay" onClick={() => setShowStatsModal(false)}>
          <div className="modal-content" style={{ maxWidth: '640px' }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
              <h3 style={{ margin: 0 }}>📊 Project Intelligence & Version Snapshots</h3>
              <button className="btn btn-secondary" style={{ padding: '0.2rem 0.5rem' }} onClick={() => setShowStatsModal(false)}>
                ✕
              </button>
            </div>

            {/* Metrics Grid */}
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-value">{project.steps.length}</div>
                <div className="stat-label">Total Steps</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">
                  {project.steps.filter((s) => s.code_snippet?.code).length}
                </div>
                <div className="stat-label">Code Blocks</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">
                  {project.steps.reduce((acc, s) => acc + (s.checklist?.length || 0), 0)}
                </div>
                <div className="stat-label">Checklist Tasks</div>
              </div>
            </div>

            {/* Privacy Preflight Banner */}
            <div style={{ marginBottom: '1.25rem' }}>
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
                <div style={{ background: 'rgba(34, 197, 94, 0.1)', border: '1px solid #22c55e', borderRadius: '6px', padding: '0.6rem 0.85rem', color: '#22c55e', fontSize: '0.85rem' }}>
                  ✓ Clean Privacy Preflight: Zero secrets detected in step instructions.
                </div>
              )}
            </div>

            {/* Snapshots & History */}
            <div style={{ borderTop: '1px solid var(--border-subtle)', paddingTop: '1rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h4 style={{ fontSize: '0.95rem', margin: 0 }}>Version Snapshots</h4>
                <div style={{ display: 'flex', gap: '0.4rem' }}>
                  <input
                    type="text"
                    placeholder="Snapshot label..."
                    className="form-control"
                    style={{ padding: '0.2rem 0.5rem', fontSize: '0.8rem', width: '150px' }}
                    value={newSnapshotLabel}
                    onChange={(e) => setNewSnapshotLabel(e.target.value)}
                  />
                  <button className="btn btn-primary" style={{ padding: '0.2rem 0.6rem', fontSize: '0.8rem' }} onClick={handleCreateSnapshot}>
                    + Take Snapshot
                  </button>
                </div>
              </div>

              <div className="snapshot-list">
                {(project.snapshots || []).length === 0 ? (
                  <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.85rem', padding: '1rem 0' }}>
                    No snapshots taken yet. Capture a baseline before major edits.
                  </div>
                ) : (
                  project.snapshots!.map((snap) => {
                    const stepDiff = project.steps.length - snap.step_count;
                    return (
                      <div key={snap.id} className="snapshot-card">
                        <div>
                          <strong>{snap.label}</strong>
                          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                            {new Date(snap.created_at).toLocaleString()} • {snap.step_count} step(s)
                          </div>
                        </div>
                        <span
                          className="timestamp-pill"
                          style={{
                            color: stepDiff >= 0 ? '#22c55e' : '#ef4444',
                            background: stepDiff >= 0 ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)',
                          }}
                        >
                          {stepDiff >= 0 ? `+${stepDiff}` : `${stepDiff}`} steps vs current
                        </span>
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          </div>
        </div>
      )}

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
                <input
                  type="radio"
                  name="format"
                  checked={selectedExportFormat === 'html'}
                  onChange={() => setSelectedExportFormat('html')}
                />
                <div>
                  <strong>Standalone Responsive HTML 2.0 Studio</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Live search, presentation slide mode, text-to-speech audio, and checklist persistence
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input
                  type="radio"
                  name="format"
                  checked={selectedExportFormat === 'markdown'}
                  onChange={() => setSelectedExportFormat('markdown')}
                />
                <div>
                  <strong>Markdown + Local Assets (README.md)</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    GitHub Flavored Markdown with code snippets & relative image links
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input
                  type="radio"
                  name="format"
                  checked={selectedExportFormat === 'json'}
                  onChange={() => setSelectedExportFormat('json')}
                />
                <div>
                  <strong>Machine-Readable JSON Archive (guide.json)</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Full schema representation for CI/CD automation & API consumption
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input
                  type="radio"
                  name="format"
                  checked={selectedExportFormat === 'gif'}
                  onChange={() => setSelectedExportFormat('gif')}
                />
                <div>
                  <strong>Animated Walkthrough GIF (walkthrough.gif)</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                    Optimized animated loop using native Rust + FFmpeg palettegen
                  </div>
                </div>
              </label>

              <label className="export-option-label">
                <input
                  type="radio"
                  name="format"
                  checked={selectedExportFormat === 'all'}
                  onChange={() => setSelectedExportFormat('all')}
                />
                <div>
                  <strong>Complete Beast Bundle (All Formats)</strong>
                  <div style={{ fontSize: '0.8rem', color: 'var(--primary)' }}>
                    Generates HTML 2.0 + Markdown + JSON + Animated GIF
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
                <button className="btn btn-primary" onClick={() => handleExport(selectedExportFormat)}>
                  Export Guide Now
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
