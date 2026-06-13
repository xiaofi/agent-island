import type { CSSProperties, ReactNode } from "react";
import { AbsoluteFill, Easing, interpolate, Sequence, useCurrentFrame, useVideoConfig } from "remotion";

const palette = {
  bg: "#080a0f",
  panel: "rgba(15, 18, 25, 0.78)",
  panelStrong: "rgba(20, 24, 33, 0.9)",
  border: "rgba(255, 255, 255, 0.14)",
  text: "rgba(248, 251, 255, 0.96)",
  muted: "rgba(218, 227, 239, 0.66)",
  faint: "rgba(218, 227, 239, 0.38)",
  cyan: "#42bff5",
  amber: "#f6b946",
  coral: "#ff7f50",
  green: "#55d68b",
  red: "#ff5f6c",
  violet: "#8f7cff",
};

const tasks = [
  {
    source: "Codex",
    status: "tool",
    statusText: "正在执行工具",
    title: "实现 Agent Island MVP",
    project: "agent-island",
    action: "写入 Vue 组件",
    time: "07:12",
  },
  {
    source: "Claude",
    status: "waiting",
    statusText: "等待用户",
    title: "检查 API server 测试失败",
    project: "api-server",
    action: "等待命令确认",
    time: "16:20",
  },
  {
    source: "Codex",
    status: "thinking",
    statusText: "正在思考",
    title: "整理 adapter discovery 文档",
    project: "agent-island",
    action: "归纳诊断字段",
    time: "05:20",
  },
  {
    source: "Manual",
    status: "completed",
    statusText: "已完成",
    title: "打包前检查清单",
    project: "agent-island",
    action: "mock 数据验证完成",
    time: "36:40",
  },
];

export const AgentIslandPromo = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <AbsoluteFill style={styles.root}>
      <TechBackground frame={frame} fps={fps} />
      <BrandChrome frame={frame} />

      <Sequence durationInFrames={6 * fps} premountFor={fps}>
        <IntroScene />
      </Sequence>
      <Sequence from={5 * fps} durationInFrames={7 * fps} premountFor={fps}>
        <ContextScene />
      </Sequence>
      <Sequence from={11 * fps} durationInFrames={8 * fps} premountFor={fps}>
        <IslandScene />
      </Sequence>
      <Sequence from={18 * fps} durationInFrames={7 * fps} premountFor={fps}>
        <PrivacyScene />
      </Sequence>
      <Sequence from={24 * fps} durationInFrames={6 * fps} premountFor={fps}>
        <OutroScene />
      </Sequence>
    </AbsoluteFill>
  );
};

function IntroScene() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = progress(frame, 0, 1.1 * fps);
  const glow = progress(frame, 0.7 * fps, 1.5 * fps);
  const exit = progress(frame, 4.7 * fps, 0.8 * fps);
  const visible = enter * (1 - exit);

  return (
    <AbsoluteFill style={{ opacity: visible }}>
      <div
        style={{
          ...styles.heroBlock,
          transform: `translateY(${interpolate(enter, [0, 1], [34, 0])}px) scale(${interpolate(glow, [0, 1], [0.98, 1])})`,
        }}
      >
        <Kicker>macOS-first local status layer</Kicker>
        <h1 style={styles.heroTitle}>Agent Island</h1>
        <p style={styles.heroCopy}>把 Codex / Claude Code 的运行状态，放在桌面边缘。</p>
      </div>
      <div
        style={{
          ...styles.heroIslandWrap,
          transform: `translate(-50%, ${interpolate(enter, [0, 1], [64, 0])}px)`,
        }}
      >
        <IslandPill mode="single" pulseFrame={frame} />
      </div>
    </AbsoluteFill>
  );
}

function ContextScene() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = progress(frame, 0, 0.8 * fps);
  const exit = progress(frame, 5.8 * fps, 0.8 * fps);
  const visible = enter * (1 - exit);

  return (
    <AbsoluteFill style={{ opacity: visible }}>
      <SceneCopy
        kicker="多 agent 并行"
        title="不用反复切回每个终端窗口"
        body="悬浮岛只展示当前需要关注的状态：运行中、执行工具、等待确认、已完成。"
        style={{
          left: 150,
          top: 168,
          transform: `translateX(${interpolate(enter, [0, 1], [-36, 0])}px)`,
        }}
      />
      <TerminalDeck frame={frame} />
      <div style={{ ...styles.statusRail, transform: `translateY(${interpolate(enter, [0, 1], [30, 0])}px)` }}>
        <StatusChip status="running" label="running" />
        <StatusChip status="tool" label="tool-running" />
        <StatusChip status="waiting" label="waiting-user" />
        <StatusChip status="completed" label="completed" />
      </div>
    </AbsoluteFill>
  );
}

function IslandScene() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = progress(frame, 0, 0.8 * fps);
  const exit = progress(frame, 6.9 * fps, 0.8 * fps);
  const visible = enter * (1 - exit);
  const expand = progress(frame, 1.4 * fps, 1.1 * fps);
  const detail = progress(frame, 4.3 * fps, 1 * fps);

  return (
    <AbsoluteFill style={{ opacity: visible }}>
      <SceneCopy
        kicker="轻量浏览"
        title="从一个小岛，展开到任务列表"
        body="点击查看来源、项目、最近动作和等待原因；完整设置和诊断进入独立窗口。"
        style={{ left: 145, top: 144 }}
      />

      <div
        style={{
          ...styles.productStage,
          transform: `translateY(${interpolate(enter, [0, 1], [48, 0])}px)`,
        }}
      >
        <IslandPill mode={expand > 0.18 ? "stacked" : "single"} pulseFrame={frame} />
        <div
          style={{
            ...styles.expandedPanel,
            opacity: expand,
            transform: `translateY(${interpolate(expand, [0, 1], [-22, 0])}px) scale(${interpolate(expand, [0, 1], [0.98, 1])})`,
          }}
        >
          <div style={styles.panelHeader}>
            <div>
              <div style={styles.panelEyebrow}>Agent Island</div>
              <div style={styles.panelTitle}>当前 agent 会话</div>
            </div>
            <div style={styles.panelIcons}>
              <span>诊断</span>
              <span>设置</span>
            </div>
          </div>

          <div style={{ display: detail < 0.5 ? "block" : "none" }}>
            {tasks.map((task, index) => (
              <TaskCard key={task.title} task={task} active={index === 1} delay={index * 5} />
            ))}
          </div>
          <div
            style={{
              ...styles.detailCard,
              opacity: detail,
              transform: `translateX(${interpolate(detail, [0, 1], [36, 0])}px)`,
              display: detail > 0.08 ? "block" : "none",
            }}
          >
            <div style={styles.detailTop}>
              <StatusDot status="waiting" />
              <span>Claude Code</span>
              <span style={styles.detailState}>等待用户</span>
            </div>
            <h3 style={styles.detailTitle}>检查 API server 测试失败</h3>
            <p style={styles.detailPath}>/Users/spf/project/api-server</p>
            <p style={styles.notice}>需要批准运行数据库迁移测试</p>
            <div style={styles.actionGrid}>
              <ActionButton label="打开任务" />
              <ActionButton label="打开目录" />
              <ActionButton label="复制摘要" />
              <ActionButton label="清除状态" />
            </div>
            <EventList />
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
}

function PrivacyScene() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = progress(frame, 0, 0.8 * fps);
  const exit = progress(frame, 5.8 * fps, 0.7 * fps);
  const visible = enter * (1 - exit);
  const pulse = Math.sin(frame / 11) * 0.5 + 0.5;

  return (
    <AbsoluteFill style={{ opacity: visible }}>
      <SceneCopy
        kicker="旁路观测"
        title="只做状态发现、归一化和展示"
        body="按来源接入官方 hook；事件写入本地状态队列，再由悬浮岛展示。"
        style={{ left: 145, top: 150 }}
      />
      <div style={styles.pipeline}>
        <PipelineNode title="Claude Code / Codex" body="官方 hook 事件" active={pulse} />
        <PipelineArrow />
        <PipelineNode title="agent-island-hook" body="最小字段过滤" active={1 - pulse} />
        <PipelineArrow />
        <PipelineNode title="本地 JSONL spool" body="append-only 状态队列" active={pulse} />
        <PipelineArrow />
        <PipelineNode title="Vue / Pinia UI" body="悬浮岛展示" active={1 - pulse} />
      </div>
      <div style={styles.guardrails}>
        <Guardrail>不接管 agent 执行流程</Guardrail>
        <Guardrail>不默认读取完整对话内容</Guardrail>
        <Guardrail>不向云端上传数据</Guardrail>
      </div>
    </AbsoluteFill>
  );
}

function OutroScene() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = progress(frame, 0, 0.9 * fps);
  const endGlow = progress(frame, 3.6 * fps, 1.1 * fps);

  return (
    <AbsoluteFill style={{ opacity: enter }}>
      <div style={styles.outroCenter}>
        <Kicker>Agent Island</Kicker>
        <h2 style={styles.outroTitle}>让 agent 状态留在你的视线边缘</h2>
        <p style={styles.outroCopy}>本地运行。轻量展示。需要关注时再提醒你。</p>
        <div
          style={{
            ...styles.outroLine,
            transform: `scaleX(${interpolate(endGlow, [0, 1], [0.2, 1])})`,
            opacity: interpolate(endGlow, [0, 1], [0.35, 1]),
          }}
        />
      </div>
    </AbsoluteFill>
  );
}

function TechBackground({ frame, fps }: { frame: number; fps: number }) {
  const sweep = (frame % (fps * 6)) / (fps * 6);
  const gridShift = frame * 0.35;

  return (
    <AbsoluteFill style={styles.background}>
      <div style={styles.glowCyan} />
      <div style={styles.glowGreen} />
      <div style={styles.glowCoral} />
      <div
        style={{
          ...styles.grid,
          backgroundPosition: `${gridShift}px ${gridShift * 0.5}px`,
        }}
      />
      <div
        style={{
          ...styles.scanLine,
          transform: `translateX(${interpolate(sweep, [0, 1], [-420, 2260])}px) rotate(14deg)`,
        }}
      />
      <ParticleField frame={frame} />
    </AbsoluteFill>
  );
}

function ParticleField({ frame }: { frame: number }) {
  const dots = Array.from({ length: 34 }, (_, index) => index);

  return (
    <AbsoluteFill>
      {dots.map((dot) => {
        const x = (dot * 173) % 1880;
        const y = 70 + ((dot * 91) % 880);
        const drift = Math.sin((frame + dot * 17) / 36) * 18;
        const opacity = 0.18 + (Math.sin((frame + dot * 13) / 23) * 0.5 + 0.5) * 0.42;
        const color = dot % 5 === 0 ? palette.green : dot % 4 === 0 ? palette.coral : palette.cyan;

        return (
          <span
            key={dot}
            style={{
              ...styles.particle,
              left: x,
              top: y,
              opacity,
              background: color,
              transform: `translate(${drift}px, ${drift * 0.25}px)`,
            }}
          />
        );
      })}
    </AbsoluteFill>
  );
}

function BrandChrome({ frame }: { frame: number }) {
  const opacity = interpolate(frame, [0, 28], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.bezier(0.16, 1, 0.3, 1),
  });

  return (
    <div style={{ ...styles.brandChrome, opacity }}>
      <div style={styles.brandLockup}>
        <span style={styles.logoMark}>AI</span>
        <span>Agent Island</span>
      </div>
      <div style={styles.chromeMeta}>local · desktop · status</div>
    </div>
  );
}

function TerminalDeck({ frame }: { frame: number }) {
  const { fps } = useVideoConfig();
  const items = [
    { title: "Codex", code: ["读取 src/components/island", "更新 TaskCard.vue", "运行 npm test -- --run"], color: palette.cyan },
    { title: "Claude Code", code: ["检查 api-server 测试", "定位 migration 失败", "等待命令确认"], color: palette.coral },
    { title: "Codex", code: ["整理 hook-ingestion.md", "归纳状态映射", "生成诊断摘要"], color: palette.green },
  ];

  return (
    <div style={styles.terminalDeck}>
      {items.map((item, index) => {
        const enter = progress(frame, (0.5 + index * 0.35) * fps, 0.7 * fps);
        return (
          <div
            key={item.title + index}
            style={{
              ...styles.terminal,
              borderColor: colorAlpha(item.color, 0.28),
              transform: `translate(${index * 38}px, ${index * 70 + interpolate(enter, [0, 1], [40, 0])}px) rotate(${index * -2.5}deg)`,
              opacity: enter,
            }}
          >
            <div style={styles.terminalTop}>
              <span style={{ ...styles.terminalDot, background: item.color }} />
              <span>{item.title}</span>
            </div>
            {item.code.map((line) => (
              <div key={line} style={styles.codeLine}>
                <span style={{ color: item.color }}>$</span>
                <span>{line}</span>
              </div>
            ))}
          </div>
        );
      })}
    </div>
  );
}

function IslandPill({ mode, pulseFrame }: { mode: "single" | "stacked"; pulseFrame: number }) {
  const pulse = Math.sin(pulseFrame / 10) * 0.5 + 0.5;

  if (mode === "stacked") {
    return (
      <div style={{ ...styles.islandPill, ...styles.islandStack }}>
        <IslandRow status="waiting" text="Claude · 等待用户 · 检查 API server 测试失败" />
        <IslandRow status="completed" text="Manual · 已完成 · 打包前检查清单" />
        <div style={styles.islandDivider} />
        <IslandRow status="running" text="2 个任务进行中" action="显示全部任务" />
      </div>
    );
  }

  return (
    <div
      style={{
        ...styles.islandPill,
        boxShadow: `0 0 ${32 + pulse * 18}px rgba(66,191,245,${0.16 + pulse * 0.12})`,
      }}
    >
      <IslandRow status="tool" text="Codex · 正在执行工具 · 写入 Vue 组件" action="显示全部任务" />
    </div>
  );
}

function IslandRow({ status, text, action }: { status: string; text: string; action?: string }) {
  return (
    <div style={styles.islandRow}>
      <div style={styles.islandLeft}>
        <StatusDot status={status} />
        <span style={styles.islandText}>{text}</span>
      </div>
      {action ? <span style={styles.islandAction}>{action}</span> : null}
    </div>
  );
}

function TaskCard({ task, active, delay }: { task: (typeof tasks)[number]; active: boolean; delay: number }) {
  const frame = useCurrentFrame();
  const enter = progress(frame, delay, 22);

  return (
    <div
      style={{
        ...styles.taskCard,
        borderColor: active ? colorAlpha(palette.coral, 0.42) : palette.border,
        background: active ? "rgba(255, 127, 80, 0.11)" : "rgba(255, 255, 255, 0.055)",
        opacity: enter,
        transform: `translateY(${interpolate(enter, [0, 1], [18, 0])}px)`,
      }}
    >
      <div style={styles.taskTop}>
        <div style={styles.source}>
          <StatusDot status={task.status} />
          <span>{task.source}</span>
        </div>
        <span style={styles.taskState}>{task.statusText}</span>
      </div>
      <div style={styles.taskTitle}>{task.title}</div>
      <div style={styles.taskMeta}>
        <span>{task.project}</span>
        <span>{task.action}</span>
        <span>{task.time}</span>
      </div>
    </div>
  );
}

function PipelineNode({ title, body, active }: { title: string; body: string; active: number }) {
  return (
    <div
      style={{
        ...styles.pipelineNode,
        boxShadow: `0 0 ${20 + active * 30}px rgba(66,191,245,${0.08 + active * 0.12})`,
      }}
    >
      <div style={styles.pipelineNodeTitle}>{title}</div>
      <div style={styles.pipelineNodeBody}>{body}</div>
    </div>
  );
}

function PipelineArrow() {
  return (
    <div style={styles.pipelineArrow}>
      <span />
    </div>
  );
}

function Guardrail({ children }: { children: ReactNode }) {
  return (
    <div style={styles.guardrail}>
      <span style={styles.checkMark}>✓</span>
      <span>{children}</span>
    </div>
  );
}

function EventList() {
  const events = ["读取测试日志", "定位失败用例", "等待用户确认命令"];

  return (
    <div style={styles.eventList}>
      <div style={styles.eventHeading}>最近事件</div>
      {events.map((event, index) => (
        <div key={event} style={styles.eventRow}>
          <time>{`10:${20 + index * 2}`}</time>
          <span>{event}</span>
        </div>
      ))}
    </div>
  );
}

function ActionButton({ label }: { label: string }) {
  return <div style={styles.actionButton}>{label}</div>;
}

function StatusChip({ status, label }: { status: string; label: string }) {
  return (
    <div style={styles.statusChip}>
      <StatusDot status={status} />
      <span>{label}</span>
    </div>
  );
}

function StatusDot({ status }: { status: string }) {
  const color =
    status === "waiting"
      ? palette.coral
      : status === "tool"
        ? palette.amber
        : status === "completed"
          ? palette.green
          : status === "failed"
            ? palette.red
            : palette.cyan;

  return (
    <span
      style={{
        ...styles.statusDot,
        background: color,
        boxShadow: `0 0 18px ${colorAlpha(color, 0.55)}`,
      }}
    />
  );
}

function SceneCopy({
  kicker,
  title,
  body,
  style,
}: {
  kicker: string;
  title: string;
  body: string;
  style?: CSSProperties;
}) {
  const frame = useCurrentFrame();
  const reveal = progress(frame, 8, 28);

  return (
    <div style={{ ...styles.sceneCopy, ...style, opacity: reveal }}>
      <Kicker>{kicker}</Kicker>
      <h2 style={styles.sceneTitle}>{title}</h2>
      <p style={styles.sceneBody}>{body}</p>
    </div>
  );
}

function Kicker({ children }: { children: ReactNode }) {
  return <div style={styles.kicker}>{children}</div>;
}

function progress(frame: number, start: number, duration: number) {
  return interpolate(frame, [start, start + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.bezier(0.16, 1, 0.3, 1),
  });
}

function colorAlpha(hex: string, alpha: number) {
  const value = hex.replace("#", "");
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

const styles: Record<string, CSSProperties> = {
  root: {
    overflow: "hidden",
    background: palette.bg,
    color: palette.text,
    fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif",
    letterSpacing: 0,
  },
  background: {
    background:
      "radial-gradient(circle at 24% 20%, rgba(66,191,245,0.18), transparent 28%), radial-gradient(circle at 76% 26%, rgba(85,214,139,0.12), transparent 28%), linear-gradient(135deg, #080a0f 0%, #10131a 46%, #07090d 100%)",
  },
  glowCyan: {
    position: "absolute",
    width: 760,
    height: 760,
    left: -180,
    top: -190,
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(66,191,245,0.22), transparent 65%)",
    filter: "blur(12px)",
  },
  glowGreen: {
    position: "absolute",
    width: 700,
    height: 700,
    right: -140,
    top: 120,
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(85,214,139,0.16), transparent 68%)",
    filter: "blur(16px)",
  },
  glowCoral: {
    position: "absolute",
    width: 520,
    height: 520,
    right: 210,
    bottom: -230,
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(255,127,80,0.12), transparent 70%)",
    filter: "blur(18px)",
  },
  grid: {
    position: "absolute",
    inset: 0,
    opacity: 0.24,
    backgroundImage:
      "linear-gradient(rgba(255,255,255,0.055) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.055) 1px, transparent 1px)",
    backgroundSize: "72px 72px",
    maskImage: "linear-gradient(to bottom, transparent 0%, black 14%, black 82%, transparent 100%)",
  },
  scanLine: {
    position: "absolute",
    top: -220,
    width: 220,
    height: 1520,
    background: "linear-gradient(90deg, transparent, rgba(66,191,245,0.16), transparent)",
    filter: "blur(8px)",
  },
  particle: {
    position: "absolute",
    width: 3,
    height: 3,
    borderRadius: "50%",
  },
  brandChrome: {
    position: "absolute",
    left: 58,
    right: 58,
    top: 44,
    zIndex: 20,
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    color: palette.muted,
    fontSize: 18,
    fontWeight: 700,
  },
  brandLockup: {
    display: "flex",
    alignItems: "center",
    gap: 12,
  },
  logoMark: {
    display: "grid",
    width: 40,
    height: 40,
    placeItems: "center",
    border: `1px solid ${colorAlpha(palette.cyan, 0.34)}`,
    borderRadius: 10,
    background: "rgba(66,191,245,0.1)",
    color: palette.cyan,
    fontSize: 13,
    fontWeight: 900,
  },
  chromeMeta: {
    fontSize: 14,
    color: palette.faint,
  },
  heroBlock: {
    position: "absolute",
    left: 150,
    top: 210,
    width: 980,
  },
  heroTitle: {
    margin: "18px 0 0",
    fontSize: 148,
    lineHeight: 0.92,
    fontWeight: 850,
  },
  heroCopy: {
    width: 720,
    margin: "30px 0 0",
    color: palette.muted,
    fontSize: 35,
    lineHeight: 1.28,
    fontWeight: 500,
  },
  heroIslandWrap: {
    position: "absolute",
    left: "50%",
    bottom: 170,
    width: 620,
  },
  kicker: {
    color: palette.cyan,
    fontSize: 19,
    lineHeight: 1,
    fontWeight: 850,
    textTransform: "uppercase",
  },
  sceneCopy: {
    position: "absolute",
    zIndex: 10,
    width: 640,
  },
  sceneTitle: {
    margin: "18px 0 0",
    fontSize: 72,
    lineHeight: 1.03,
    fontWeight: 840,
  },
  sceneBody: {
    margin: "24px 0 0",
    color: palette.muted,
    fontSize: 29,
    lineHeight: 1.36,
    fontWeight: 500,
  },
  terminalDeck: {
    position: "absolute",
    right: 260,
    top: 130,
    width: 700,
    height: 720,
  },
  terminal: {
    position: "absolute",
    width: 590,
    minHeight: 240,
    padding: "24px 26px",
    border: "1px solid",
    borderRadius: 18,
    background: "rgba(8, 10, 15, 0.78)",
    boxShadow: "0 34px 80px rgba(0,0,0,0.34)",
    backdropFilter: "blur(20px)",
  },
  terminalTop: {
    display: "flex",
    alignItems: "center",
    gap: 10,
    marginBottom: 22,
    color: palette.text,
    fontSize: 19,
    fontWeight: 800,
  },
  terminalDot: {
    width: 10,
    height: 10,
    borderRadius: "50%",
  },
  codeLine: {
    display: "flex",
    gap: 13,
    color: palette.muted,
    fontFamily: "SFMono-Regular, ui-monospace, Menlo, monospace",
    fontSize: 21,
    lineHeight: 1.8,
  },
  statusRail: {
    position: "absolute",
    left: 150,
    bottom: 150,
    display: "flex",
    gap: 14,
  },
  statusChip: {
    display: "flex",
    alignItems: "center",
    gap: 10,
    padding: "12px 16px",
    border: `1px solid ${palette.border}`,
    borderRadius: 999,
    background: "rgba(255,255,255,0.06)",
    color: palette.muted,
    fontSize: 17,
    fontWeight: 750,
  },
  productStage: {
    position: "absolute",
    right: 155,
    top: 130,
    width: 760,
  },
  islandPill: {
    position: "relative",
    zIndex: 2,
    width: 620,
    minHeight: 72,
    padding: "0 20px",
    border: `1px solid ${palette.border}`,
    borderRadius: 999,
    background: "rgba(12, 14, 18, 0.92)",
    boxShadow: "0 28px 78px rgba(0,0,0,0.42)",
    backdropFilter: "blur(22px) saturate(1.2)",
  },
  islandStack: {
    minHeight: 190,
    padding: "14px 20px",
    borderRadius: 28,
  },
  islandRow: {
    display: "flex",
    minHeight: 54,
    alignItems: "center",
    justifyContent: "space-between",
    gap: 16,
  },
  islandLeft: {
    display: "flex",
    minWidth: 0,
    alignItems: "center",
    gap: 12,
  },
  islandText: {
    overflow: "hidden",
    color: palette.text,
    fontSize: 19,
    fontWeight: 760,
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  islandAction: {
    flex: "none",
    color: palette.muted,
    fontSize: 15,
    fontWeight: 750,
  },
  islandDivider: {
    height: 1,
    background: "rgba(255,255,255,0.1)",
  },
  expandedPanel: {
    width: 720,
    minHeight: 620,
    marginTop: 18,
    padding: 20,
    border: `1px solid ${palette.border}`,
    borderRadius: 26,
    background: "linear-gradient(180deg, rgba(255,255,255,0.07), rgba(255,255,255,0.025)), rgba(12,14,18,0.92)",
    boxShadow: "0 36px 100px rgba(0,0,0,0.42)",
    backdropFilter: "blur(22px) saturate(1.2)",
  },
  panelHeader: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    marginBottom: 17,
  },
  panelEyebrow: {
    color: palette.faint,
    fontSize: 14,
    fontWeight: 850,
    textTransform: "uppercase",
  },
  panelTitle: {
    marginTop: 4,
    fontSize: 24,
    fontWeight: 850,
  },
  panelIcons: {
    display: "flex",
    gap: 10,
    color: palette.muted,
    fontSize: 14,
    fontWeight: 750,
  },
  taskCard: {
    marginBottom: 12,
    padding: "17px 18px",
    border: "1px solid",
    borderRadius: 14,
  },
  taskTop: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    color: palette.muted,
    fontSize: 15,
    fontWeight: 800,
  },
  source: {
    display: "flex",
    alignItems: "center",
    gap: 10,
  },
  taskState: {
    color: palette.faint,
  },
  taskTitle: {
    marginTop: 12,
    fontSize: 23,
    fontWeight: 820,
  },
  taskMeta: {
    display: "flex",
    gap: 18,
    marginTop: 12,
    color: palette.muted,
    fontSize: 15,
    fontWeight: 620,
  },
  detailCard: {
    padding: 22,
    border: `1px solid ${colorAlpha(palette.coral, 0.34)}`,
    borderRadius: 18,
    background: "rgba(255,255,255,0.055)",
  },
  detailTop: {
    display: "flex",
    alignItems: "center",
    gap: 10,
    color: palette.muted,
    fontSize: 16,
    fontWeight: 800,
  },
  detailState: {
    marginLeft: "auto",
    color: palette.coral,
  },
  detailTitle: {
    margin: "22px 0 0",
    fontSize: 32,
    lineHeight: 1.14,
  },
  detailPath: {
    margin: "12px 0 0",
    color: palette.faint,
    fontSize: 16,
  },
  notice: {
    margin: "18px 0 0",
    padding: "14px 16px",
    borderRadius: 12,
    background: "rgba(255,127,80,0.12)",
    color: "#ffd3c4",
    fontSize: 17,
    fontWeight: 700,
  },
  actionGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(4, 1fr)",
    gap: 10,
    marginTop: 18,
  },
  actionButton: {
    display: "grid",
    minHeight: 44,
    placeItems: "center",
    border: `1px solid ${palette.border}`,
    borderRadius: 12,
    background: "rgba(255,255,255,0.055)",
    color: palette.muted,
    fontSize: 14,
    fontWeight: 760,
  },
  eventList: {
    marginTop: 22,
  },
  eventHeading: {
    marginBottom: 8,
    color: palette.faint,
    fontSize: 14,
    fontWeight: 850,
  },
  eventRow: {
    display: "grid",
    gridTemplateColumns: "76px 1fr",
    padding: "9px 0",
    color: palette.muted,
    fontSize: 15,
    borderTop: "1px solid rgba(255,255,255,0.08)",
  },
  pipeline: {
    position: "absolute",
    left: 145,
    right: 145,
    top: 505,
    display: "grid",
    gridTemplateColumns: "1fr 70px 1fr 70px 1fr 70px 1fr",
    alignItems: "center",
    gap: 10,
  },
  pipelineNode: {
    minHeight: 150,
    padding: "28px 26px",
    border: `1px solid ${palette.border}`,
    borderRadius: 18,
    background: "rgba(12,14,18,0.74)",
    backdropFilter: "blur(18px)",
  },
  pipelineNodeTitle: {
    fontSize: 24,
    fontWeight: 850,
  },
  pipelineNodeBody: {
    marginTop: 12,
    color: palette.muted,
    fontSize: 17,
    lineHeight: 1.35,
    fontWeight: 620,
  },
  pipelineArrow: {
    height: 2,
    background: `linear-gradient(90deg, ${colorAlpha(palette.cyan, 0.18)}, ${colorAlpha(palette.cyan, 0.82)})`,
    position: "relative",
  },
  guardrails: {
    position: "absolute",
    left: 145,
    right: 145,
    bottom: 145,
    display: "grid",
    gridTemplateColumns: "repeat(3, 1fr)",
    gap: 16,
  },
  guardrail: {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "18px 20px",
    border: `1px solid ${colorAlpha(palette.green, 0.24)}`,
    borderRadius: 14,
    background: "rgba(85,214,139,0.075)",
    color: palette.text,
    fontSize: 19,
    fontWeight: 760,
  },
  checkMark: {
    color: palette.green,
    fontWeight: 900,
  },
  outroCenter: {
    position: "absolute",
    left: "50%",
    top: "50%",
    width: 1180,
    transform: "translate(-50%, -50%)",
    textAlign: "center",
  },
  outroTitle: {
    margin: "22px 0 0",
    fontSize: 94,
    lineHeight: 1.02,
    fontWeight: 860,
  },
  outroCopy: {
    margin: "26px auto 0",
    width: 760,
    color: palette.muted,
    fontSize: 30,
    lineHeight: 1.34,
    fontWeight: 520,
  },
  outroLine: {
    width: 720,
    height: 2,
    margin: "46px auto 0",
    background: `linear-gradient(90deg, transparent, ${palette.cyan}, ${palette.green}, transparent)`,
    transformOrigin: "center",
  },
  statusDot: {
    width: 10,
    height: 10,
    flex: "none",
    borderRadius: "50%",
  },
};
