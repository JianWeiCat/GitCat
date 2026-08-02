import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, SearchBar } from "@lobehub/ui";
import {
  ArrowLeft, ArrowRight, BarChart3, Check, ChevronRight, Clock3, Download,
  FileCode2, Folder, GitBranch, GitCommitHorizontal, GitFork, Layers3, Plus,
  RefreshCw, ScanSearch, Search, Trash2, Upload, type LucideIcon,
} from "lucide-react";
import "./App.css";

type View = "discover" | "repository" | "bulk" | "activity";
type RepoTab = "changes" | "history" | "branches" | "insights";
type Operation = "status" | "pull" | "gc";
type SyncAction = "fetch" | "pull" | "push";

interface Overview { path: string; total_commits: number; total_branches: number; total_tags: number; contributors: number; head_branch: string; last_commit_date: string; }
interface Branch { name: string; is_head: boolean; commit_hash: string; commit_message: string; commit_date: string; author: string; is_protected: boolean; can_delete: boolean; }
interface GraphNode { hash: string; parents: string[]; refs: string[]; author: string; date: string; message: string; }
interface Language { language: string; bytes: number; files: number; }
interface Author { name: string; commits: number; lines_added: number; code_lines: number; }
interface Stats { authors: Author[]; }
interface RepoStatus { branch: string; modified: number; staged: number; untracked: number; conflicted: number; ahead: number; behind: number; }
interface WorkingFile { path: string; index_status: string; worktree_status: string; staged: boolean; modified: boolean; untracked: boolean; conflicted: boolean; }
interface FileChange { path: string; added: number; deleted: number; }
interface CommitFiles { files_changed: number; insertions: number; deletions: number; files: FileChange[]; }
interface ScanResponse { repos: string[]; roots: string[]; }
interface TaskResult { repo_path: string; success: boolean; message: string; }
interface Run { id: string; operation: Operation; results: TaskResult[]; time: string; }
interface SelectedFile { file: WorkingFile; staged: boolean; }
interface GraphEdge { from: number; to: number; half?: "top" | "bottom"; }
interface GraphRowData { node: GraphNode; lane: number; edges: GraphEdge[]; lanes: number; }

const nameOf = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() || path;
const formatBytes = (bytes: number) => bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;
const shortRef = (ref: string) => ref.replace("HEAD -> ", "").replace("origin/", "↗ ");

export default function App() {
  const [view, setView] = useState<View>("discover");
  const [repos, setRepos] = useState<string[]>([]);
  const [roots, setRoots] = useState<string[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [repoPath, setRepoPath] = useState<string | null>(null);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [branches, setBranches] = useState<Branch[]>([]);
  const [graph, setGraph] = useState<GraphNode[]>([]);
  const [activeCommit, setActiveCommit] = useState<GraphNode | null>(null);
  const [commitFiles, setCommitFiles] = useState<CommitFiles | null>(null);
  const [repoStatus, setRepoStatus] = useState<RepoStatus | null>(null);
  const [workingFiles, setWorkingFiles] = useState<WorkingFile[]>([]);
  const [selectedFile, setSelectedFile] = useState<SelectedFile | null>(null);
  const [fileDiff, setFileDiff] = useState("");
  const [commitMessage, setCommitMessage] = useState("");
  const [tab, setTab] = useState<RepoTab>("changes");
  const [languages, setLanguages] = useState<Language[] | null>(null);
  const [stats, setStats] = useState<Stats | null>(null);
  const [analyticsLoading, setAnalyticsLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [operation, setOperation] = useState<Operation>("status");
  const [runs, setRuns] = useState<Run[]>([]);

  const visibleRepos = useMemo(() => repos.filter((path) => path.toLowerCase().includes(query.toLowerCase())), [repos, query]);
  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!repoPath || !activeCommit) { setCommitFiles(null); return; }
    setCommitFiles(null);
    void invoke<CommitFiles>("get_commit_files", { path: repoPath, commitHash: activeCommit.hash })
      .then(setCommitFiles)
      .catch(() => setCommitFiles({ files_changed: 0, insertions: 0, deletions: 0, files: [] }));
  }, [repoPath, activeCommit]);

  const inform = (message: string) => setToast(message);
  const scan = async (command: "scan_all_disks" | "scan_repos", root?: string) => {
    if (command === "scan_repos" && !root) return;
    setBusy(true);
    inform(command === "scan_all_disks" ? "正在扫描磁盘…" : `正在扫描 ${root}…`);
    try {
      const data = command === "scan_all_disks"
        ? await invoke<ScanResponse>(command, { depth: 5 })
        : await invoke<ScanResponse>(command, { path: root, depth: 6 });
      setRepos(data.repos); setRoots(data.roots); setSelected(new Set(data.repos));
      inform(`找到 ${data.repos.length} 个 Git 仓库`);
    } catch (error) { inform(`扫描失败：${String(error)}`); } finally { setBusy(false); }
  };
  const scanFolder = async () => {
    try {
      const folder = await invoke<string | null>("pick_folder");
      if (folder) await scan("scan_repos", folder);
    } catch (error) { inform(`无法选择文件夹：${String(error)}`); }
  };
  const readRepository = async (path: string) => Promise.all([
    invoke<Overview>("open_repository", { path }),
    invoke<Branch[]>("get_branches", { path }),
    invoke<GraphNode[]>("get_commit_graph", { path, maxCount: 120 }),
    invoke<RepoStatus>("get_repo_status", { path }),
    invoke<WorkingFile[]>("get_working_tree", { path }),
  ]);
  const applyRepositoryData = (path: string, data: [Overview, Branch[], GraphNode[], RepoStatus, WorkingFile[]], resetTab = false) => {
    const [nextOverview, nextBranches, nextGraph, nextStatus, nextFiles] = data;
    setRepoPath(path); setOverview(nextOverview); setBranches(nextBranches); setGraph(nextGraph);
    setActiveCommit((current) => nextGraph.find((node) => node.hash === current?.hash) || nextGraph[0] || null);
    setRepoStatus(nextStatus); setWorkingFiles(nextFiles);
    if (selectedFile && !nextFiles.some((item) => item.path === selectedFile.file.path)) { setSelectedFile(null); setFileDiff(""); }
    if (resetTab) { setLanguages(null); setStats(null); setTab("changes"); setView("repository"); }
  };
  const openRepository = async (path: string) => {
    setBusy(true); inform(`正在读取 ${nameOf(path)}…`);
    try {
      applyRepositoryData(path, await readRepository(path), true);
      inform(`已打开 ${nameOf(path)}`);
    } catch (error) { inform(`无法打开仓库：${String(error)}`); } finally { setBusy(false); }
  };
  const refreshRepository = async (message?: string) => {
    if (!repoPath) return;
    try {
      applyRepositoryData(repoPath, await readRepository(repoPath));
      if (message) inform(message);
    } catch (error) { inform(`刷新失败：${String(error)}`); }
  };
  const loadInsights = async () => {
    if (!repoPath || analyticsLoading || (languages && stats)) return;
    setAnalyticsLoading(true);
    try {
      const [nextLanguages, nextStats] = await Promise.all([
        invoke<Language[]>("get_language_stats", { path: repoPath }),
        invoke<Stats>("get_stats", { path: repoPath, author: null, since: null, until: null }),
      ]);
      setLanguages(nextLanguages); setStats(nextStats);
    } catch (error) { inform(`无法读取代码分析：${String(error)}`); } finally { setAnalyticsLoading(false); }
  };
  const changeTab = (next: RepoTab) => { setTab(next); if (next === "insights") void loadInsights(); };
  const selectWorkingFile = async (file: WorkingFile, staged: boolean) => {
    if (!repoPath) return;
    setSelectedFile({ file, staged }); setFileDiff("");
    if (file.untracked && !staged) return;
    try { setFileDiff(await invoke<string>("get_working_tree_diff", { path: repoPath, filePath: file.path, staged })); }
    catch (error) { setFileDiff(`无法读取差异：${String(error)}`); }
  };
  const changeStage = async (paths: string[], stage: boolean) => {
    if (!repoPath || !paths.length) return;
    setBusy(true);
    try {
      await invoke(stage ? "stage_files" : "unstage_files", { path: repoPath, paths });
      setSelectedFile(null); setFileDiff("");
      await refreshRepository(stage ? "已暂存所选文件" : "已移出暂存区");
    } catch (error) { inform(`${stage ? "暂存" : "取消暂存"}失败：${String(error)}`); } finally { setBusy(false); }
  };
  const commit = async () => {
    if (!repoPath || !commitMessage.trim()) return;
    setBusy(true);
    try {
      const hash = await invoke<string>("commit_staged", { path: repoPath, message: commitMessage });
      setCommitMessage(""); await refreshRepository(`提交成功 · ${hash}`);
    } catch (error) { inform(`提交失败：${String(error)}`); } finally { setBusy(false); }
  };
  const sync = async (action: SyncAction) => {
    if (!repoPath) return;
    if (action === "push" && !window.confirm("确认将当前分支推送到远端吗？")) return;
    setBusy(true); inform(`${action === "fetch" ? "获取" : action === "pull" ? "拉取" : "推送"}中…`);
    try {
      await invoke<string>("sync_repository", { path: repoPath, action });
      await refreshRepository(`${action === "fetch" ? "获取" : action === "pull" ? "拉取" : "推送"}完成`);
    } catch (error) { inform(`同步失败：${String(error)}`); } finally { setBusy(false); }
  };
  const checkoutBranch = async (branchName: string) => {
    if (!repoPath) return;
    setBusy(true);
    try { await invoke("checkout_branch", { path: repoPath, branchName }); await refreshRepository(`已切换到 ${branchName}`); }
    catch (error) { inform(`切换失败：${String(error)}`); } finally { setBusy(false); }
  };
  const createBranch = async () => {
    if (!repoPath) return;
    const branchName = window.prompt("新分支名称");
    if (!branchName?.trim()) return;
    setBusy(true);
    try { await invoke("create_branch", { path: repoPath, branchName: branchName.trim(), checkout: true }); await refreshRepository(`已创建并切换到 ${branchName.trim()}`); }
    catch (error) { inform(`创建失败：${String(error)}`); } finally { setBusy(false); }
  };
  const deleteBranch = async (branchName: string) => {
    if (!repoPath || !window.confirm(`确认安全删除本地分支 ${branchName}？未合并分支不会被删除。`)) return;
    setBusy(true);
    try { await invoke("delete_branch", { path: repoPath, branchName, force: false }); await refreshRepository(`已删除 ${branchName}`); }
    catch (error) { inform(`删除失败：${String(error)}`); } finally { setBusy(false); }
  };
  const runBulk = async () => {
    if (!selected.size) { inform("请先选择仓库"); setView("discover"); return; }
    if (operation !== "status" && !window.confirm(`确认对 ${selected.size} 个仓库执行此操作吗？`)) return;
    setBusy(true);
    try {
      const results = await invoke<TaskResult[]>("run_selected_operation", { paths: [...selected], operation });
      setRuns((previous) => [{ id: crypto.randomUUID(), operation, results, time: new Date().toLocaleString() }, ...previous]); setView("activity");
      inform(`完成 ${results.filter((item) => item.success).length}/${results.length}`);
    } catch (error) { inform(`操作失败：${String(error)}`); } finally { setBusy(false); }
  };
  const toggle = (path: string) => setSelected((previous) => { const next = new Set(previous); if (next.has(path)) next.delete(path); else next.add(path); return next; });
  const navigate = (next: View) => { setView(next); setToast(null); };

  return <div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><img className="brand-mark" src="/gitcat-logo.png" alt="" aria-hidden="true" /><span>GitCat</span></div>
      <nav aria-label="主导航">
        <Nav active={view === "discover" || view === "repository"} label="仓库工作区" icon="folder" onClick={() => navigate(repoPath ? "repository" : "discover")} />
        <Nav active={view === "bulk"} label="批量任务" icon="layers" onClick={() => navigate("bulk")} />
        <Nav active={view === "activity"} label="操作记录" icon="clock" onClick={() => navigate("activity")} />
      </nav>
      <div className="side-section"><span>最近仓库</span>{repos.slice(0, 8).map((path) => <button type="button" key={path} className={path === repoPath ? "repo-shortcut active" : "repo-shortcut"} onClick={() => void openRepository(path)} title={path}><Icon name="git" />{nameOf(path)}</button>)}{repos.length > 8 && <small>还有 {repos.length - 8} 个仓库</small>}</div>
      <p className="side-foot">本地模式</p>
    </aside>
    <section className="main-shell">
      <header className={view === "discover" ? "topbar compact" : "topbar"}>
        <div><span className="path-label">{view === "repository" && repoPath ? `仓库 / ${nameOf(repoPath)}` : view === "bulk" ? "自动化" : view === "activity" ? "审计" : "仓库"}</span>{view !== "discover" && <h1>{view === "repository" ? nameOf(repoPath || "") : view === "bulk" ? "批量任务" : "操作记录"}</h1>}</div>
        {view === "repository" && <div className="sync-actions"><button onClick={() => void sync("fetch")} disabled={busy} title="获取远端引用"><Download />获取</button><button onClick={() => void sync("pull")} disabled={busy} title="拉取当前分支"><RefreshCw />拉取</button><button onClick={() => void sync("push")} disabled={busy} title="推送当前分支"><Upload />推送{repoStatus?.ahead ? <b>{repoStatus.ahead}</b> : null}</button></div>}
        {view !== "discover" && <Button className="ghost-button repo-list-button" icon={Folder} onClick={() => navigate("discover")}>仓库列表</Button>}
      </header>
      {toast && <div className={busy ? "toast working" : "toast"} role="status"><span />{toast}</div>}
      <main className="content">
        {view === "discover" && <Discover repos={visibleRepos} roots={roots} query={query} selected={selected} busy={busy} onQuery={setQuery} onScanAll={() => void scan("scan_all_disks")} onChooseFolder={() => void scanFolder()} onOpen={openRepository} onToggle={toggle} />}
        {view === "repository" && <Repository path={repoPath} overview={overview} status={repoStatus} branches={branches} graph={graph} active={activeCommit} commitFiles={commitFiles} files={workingFiles} selectedFile={selectedFile} fileDiff={fileDiff} commitMessage={commitMessage} tab={tab} languages={languages} stats={stats} loading={analyticsLoading} busy={busy} onBack={() => navigate("discover")} onTab={changeTab} onCommit={setActiveCommit} onSelectFile={selectWorkingFile} onStage={changeStage} onCommitStaged={commit} onCommitMessage={setCommitMessage} onCheckout={checkoutBranch} onCreateBranch={createBranch} onDeleteBranch={deleteBranch} />}
        {view === "bulk" && <Bulk repos={[...selected]} operation={operation} busy={busy} onOperation={setOperation} onRun={() => void runBulk()} />}
        {view === "activity" && <Activity runs={runs} />}
      </main>
    </section>
  </div>;
}

function Discover({ repos, roots, query, selected, busy, onQuery, onScanAll, onChooseFolder, onOpen, onToggle }: { repos: string[]; roots: string[]; query: string; selected: Set<string>; busy: boolean; onQuery: (value: string) => void; onScanAll: () => void; onChooseFolder: () => void; onOpen: (path: string) => void; onToggle: (path: string) => void; }) {
  return <section className="discover-page"><div className="scan-grid"><section className="scan-card featured"><div><span className="eyebrow">全盘发现</span><h2>扫描磁盘</h2><p>查找本机 Git 仓库，集中查看状态和待处理变更。</p></div><Button type="primary" className="primary-button" icon={ScanSearch} loading={busy} onClick={onScanAll}>{busy ? "扫描中…" : "扫描磁盘"}</Button></section><section className="scan-card"><div><span className="eyebrow">指定范围</span><h2>选择文件夹</h2><p>只扫描指定目录，适合大型磁盘或固定工作区。</p></div><Button className="ghost-button" icon={Folder} disabled={busy} onClick={onChooseFolder}>选择文件夹</Button></section></div><section className="repository-list"><div className="list-head"><div><h2>{repos.length ? `发现 ${repos.length} 个仓库` : "仓库列表"}</h2><p>{roots.length ? roots.join("、") : "扫描结果会显示在这里。"}</p></div>{repos.length > 0 && <SearchBar className="search" enableShortKey={false} onInputChange={onQuery} placeholder="筛选仓库" value={query} />}</div>{repos.length ? <div className="repo-list">{repos.map((path) => <article className="repo-row" key={path}><label className="checkbox"><input type="checkbox" checked={selected.has(path)} onChange={() => onToggle(path)} aria-label={`选择 ${nameOf(path)}`} /><span /></label><button type="button" onClick={() => void onOpen(path)}><Icon name="folder" /><span><strong>{nameOf(path)}</strong><small>{path}</small></span></button><button type="button" className="open-link" onClick={() => void onOpen(path)}>打开 <Icon name="arrow" /></button></article>)}</div> : <Empty icon="scan" title="暂无仓库" text="扫描磁盘或选择文件夹开始使用。" />}</section></section>;
}

interface RepositoryProps {
  path: string | null; overview: Overview | null; status: RepoStatus | null; branches: Branch[]; graph: GraphNode[]; active: GraphNode | null; commitFiles: CommitFiles | null;
  files: WorkingFile[]; selectedFile: SelectedFile | null; fileDiff: string; commitMessage: string; tab: RepoTab; languages: Language[] | null; stats: Stats | null; loading: boolean; busy: boolean;
  onBack: () => void; onTab: (tab: RepoTab) => void; onCommit: (commit: GraphNode) => void; onSelectFile: (file: WorkingFile, staged: boolean) => void;
  onStage: (paths: string[], stage: boolean) => void; onCommitStaged: () => void; onCommitMessage: (message: string) => void; onCheckout: (branch: string) => void; onCreateBranch: () => void; onDeleteBranch: (branch: string) => void;
}

function Repository(props: RepositoryProps) {
  const { path, overview, status, branches, graph, active, commitFiles, files, selectedFile, fileDiff, commitMessage, tab, languages, stats, loading, busy } = props;
  if (!path || !overview) return <Empty icon="folder" title="选择一个仓库" text="从仓库列表打开一个 Git 仓库。" />;
  const changes = (status?.modified || 0) + (status?.staged || 0) + (status?.untracked || 0);
  return <section className="repo-page">
    <button className="back-link" onClick={props.onBack}><Icon name="arrowLeft" /> 所有仓库</button>
    <div className="repo-title"><div><h2>{nameOf(path)}</h2><p>{path}</p></div><span className="branch-badge"><Icon name="git" /> {overview.head_branch}{status?.behind ? <small>↓{status.behind}</small> : null}{status?.ahead ? <small>↑{status.ahead}</small> : null}</span></div>
    <div className="repo-summary"><div><span>当前分支</span><strong>{overview.head_branch}</strong><small>{status?.ahead || status?.behind ? `领先 ${status.ahead} · 落后 ${status.behind}` : "未检测到领先或落后提交"}</small></div><div><span>最新提交</span><strong>{overview.last_commit_date || "—"}</strong><small>{graph[0]?.message || "没有可读取的提交"}</small></div><div className={status?.conflicted ? "danger-summary" : ""}><span>工作区</span><strong>{status?.conflicted ? `${status.conflicted} 个冲突` : changes ? `${changes} 处变更` : "干净"}</strong><small>{changes ? `暂存 ${status?.staged || 0} · 修改 ${status?.modified || 0} · 未跟踪 ${status?.untracked || 0}` : "没有未提交的文件"}</small></div></div>
    <div className="tabs" role="tablist"><Tab active={tab === "changes"} label={`变更 (${files.length})`} onClick={() => props.onTab("changes")} /><Tab active={tab === "history"} label="提交历史" onClick={() => props.onTab("history")} /><Tab active={tab === "branches"} label={`分支 (${branches.length})`} onClick={() => props.onTab("branches")} /><Tab active={tab === "insights"} label="仓库洞察" onClick={() => props.onTab("insights")} /></div>
    {tab === "changes" && <Changes files={files} selected={selectedFile} diff={fileDiff} message={commitMessage} busy={busy} onSelect={props.onSelectFile} onStage={props.onStage} onMessage={props.onCommitMessage} onCommit={props.onCommitStaged} />}
    {tab === "history" && <History graph={graph} active={active} details={commitFiles} onCommit={props.onCommit} />}
    {tab === "branches" && <BranchList branches={branches} busy={busy} onCheckout={props.onCheckout} onCreate={props.onCreateBranch} onDelete={props.onDeleteBranch} />}
    {tab === "insights" && <Insights languages={languages} stats={stats} loading={loading} />}
  </section>;
}

function Changes({ files, selected, diff, message, busy, onSelect, onStage, onMessage, onCommit }: { files: WorkingFile[]; selected: SelectedFile | null; diff: string; message: string; busy: boolean; onSelect: (file: WorkingFile, staged: boolean) => void; onStage: (paths: string[], stage: boolean) => void; onMessage: (message: string) => void; onCommit: () => void; }) {
  const staged = files.filter((file) => file.staged);
  const unstaged = files.filter((file) => file.modified || file.untracked || file.conflicted);
  if (!files.length) return <Empty icon="check" title="工作区干净" text="没有需要暂存或提交的变更。" />;
  return <section className="changes-layout">
    <aside className="change-tree">
      <FileGroup title="已暂存" badge={staged.length} files={staged} staged selected={selected} action="全部取消暂存" busy={busy} onSelect={onSelect} onAction={() => onStage(staged.map((file) => file.path), false)} />
      <FileGroup title="未暂存" badge={unstaged.length} files={unstaged} staged={false} selected={selected} action="全部暂存" busy={busy} onSelect={onSelect} onAction={() => onStage(unstaged.map((file) => file.path), true)} />
    </aside>
    <section className="diff-panel">
      <header><div><strong>{selected?.file.path || "选择文件查看差异"}</strong>{selected && <small>{selected.staged ? "已暂存版本与 HEAD" : selected.file.untracked ? "未跟踪文件" : "工作区与暂存区"}</small>}</div>{selected && <button className="inline-action" disabled={busy} onClick={() => onStage([selected.file.path], !selected.staged)}>{selected.staged ? "取消暂存" : "暂存文件"}</button>}</header>
      <DiffContent diff={diff} untracked={Boolean(selected?.file.untracked && !selected.staged)} />
    </section>
    <aside className="commit-composer"><div><span>提交到当前分支</span><strong>{staged.length} 个已暂存文件</strong></div><textarea aria-label="提交说明" value={message} onChange={(event) => onMessage(event.target.value)} placeholder="填写提交说明…" rows={5} /><Button type="primary" className="primary-button" icon={GitCommitHorizontal} loading={busy} disabled={!staged.length || !message.trim()} onClick={onCommit}>提交</Button><p>只提交“已暂存”区域中的文件。请先检查右侧差异。</p></aside>
  </section>;
}

function FileGroup({ title, badge, files, staged, selected, action, busy, onSelect, onAction }: { title: string; badge: number; files: WorkingFile[]; staged: boolean; selected: SelectedFile | null; action: string; busy: boolean; onSelect: (file: WorkingFile, staged: boolean) => void; onAction: () => void; }) {
  return <section className="file-group"><header><span><ChevronRight />{title}<b>{badge}</b></span>{files.length > 0 && <button disabled={busy} onClick={onAction}>{action}</button>}</header>{files.length ? files.map((file) => <button type="button" key={`${title}-${file.path}`} className={selected?.file.path === file.path && selected.staged === staged ? "file-row active" : "file-row"} disabled={busy} onClick={() => onSelect(file, staged)} title={file.path}><FileCode2 /><span>{nameOf(file.path)}<small>{file.path.includes("/") ? file.path.slice(0, file.path.lastIndexOf("/")) : "项目根目录"}</small></span><em className={file.conflicted ? "conflict" : ""}>{file.conflicted ? "!" : file.untracked ? "U" : staged ? file.index_status : file.worktree_status}</em></button>) : <p className="group-empty">没有文件</p>}</section>;
}

function DiffContent({ diff, untracked }: { diff: string; untracked: boolean; }) {
  if (untracked) return <div className="diff-empty"><FileCode2 /><p>未跟踪文件还没有 Git 差异。</p><small>暂存文件后即可查看与 HEAD 的完整差异。</small></div>;
  if (!diff) return <div className="diff-empty"><GitFork /><p>选择左侧文件查看差异。</p></div>;
  const lines = diff.split("\n").slice(0, 500);
  return <pre className="diff-code" aria-label="Git 差异">{lines.map((line, index) => <span className={line.startsWith("+") && !line.startsWith("+++") ? "added" : line.startsWith("-") && !line.startsWith("---") ? "removed" : line.startsWith("@@") ? "hunk" : ""} key={`${index}-${line.slice(0, 20)}`}>{line || " "}</span>)}</pre>;
}

function History({ graph, active, details, onCommit }: { graph: GraphNode[]; active: GraphNode | null; details: CommitFiles | null; onCommit: (commit: GraphNode) => void; }) {
  const [filter, setFilter] = useState("");
  const rows = useMemo(() => buildGraph(graph), [graph]);
  const visible = rows.filter(({ node }) => `${node.message} ${node.author} ${node.hash} ${node.refs.join(" ")}`.toLowerCase().includes(filter.toLowerCase()));
  const laneCount = Math.max(1, ...rows.map((row) => row.lanes));
  const graphWidth = Math.min(196, 28 + laneCount * 20);
  return <section className="history-workspace">
    <div className="history-panel">
      <div className="history-toolbar"><div><Search /><input aria-label="筛选提交历史" value={filter} onChange={(event) => setFilter(event.target.value)} placeholder="按提交、作者、哈希或引用筛选" /></div><span>{visible.length} 条提交</span></div>
      <div className="commit-list" style={{ "--graph-width": `${graphWidth}px` } as React.CSSProperties}>{visible.map((row) => <button type="button" className={active?.hash === row.node.hash ? "commit-row selected" : "commit-row"} key={row.node.hash} onClick={() => onCommit(row.node)}><CommitGraph row={row} width={graphWidth} /><span className="commit-message"><strong>{row.node.message}</strong><small>{row.node.author} · {row.node.date}</small></span><span className="refs">{row.node.refs.slice(0, 3).map((ref) => <em key={ref}>{shortRef(ref)}</em>)}</span><code>{row.node.hash.slice(0, 7)}</code></button>)}</div>
    </div>
    <aside className="commit-detail"><h3>提交详情</h3>{active ? <><code>{active.hash}</code><h4>{active.message}</h4><dl><div><dt>作者</dt><dd>{active.author}</dd></div><div><dt>时间</dt><dd>{active.date}</dd></div><div><dt>父提交</dt><dd>{active.parents.length ? active.parents.map((item) => item.slice(0, 7)).join("、") : "初始提交"}</dd></div></dl><div className="commit-stats"><span>文件 {details?.files_changed ?? "…"}</span><b>+{details?.insertions ?? 0}</b><em>-{details?.deletions ?? 0}</em></div><div className="changed-files">{details?.files.map((file) => <div key={file.path}><span>{file.path}</span><b>+{file.added}</b><em>-{file.deleted}</em></div>)}</div></> : <p>选择一条提交。</p>}</aside>
  </section>;
}

function buildGraph(nodes: GraphNode[]): GraphRowData[] {
  let lanes: string[] = [];
  return nodes.map((node) => {
    let lane = lanes.indexOf(node.hash);
    if (lane < 0) { lanes = [node.hash, ...lanes]; lane = 0; }
    const before = [...lanes];
    const next = [...before];
    const edges: GraphEdge[] = [];
    before.forEach((ref, index) => {
      if (index === lane) return;
      const target = next.indexOf(ref);
      if (target >= 0) edges.push({ from: index, to: target });
    });
    edges.push({ from: lane, to: lane, half: "top" });
    if (!node.parents.length) next.splice(lane, 1);
    node.parents.forEach((parent, index) => {
      let target = next.indexOf(parent);
      if (index === 0) {
        if (target < 0) { next[lane] = parent; target = lane; }
        else if (target !== lane) { next.splice(lane, 1); if (target > lane) target -= 1; }
      } else if (target < 0) { target = Math.min(lane + index, next.length); next.splice(target, 0, parent); }
      edges.push({ from: lane, to: target, half: "bottom" });
    });
    lanes = next;
    return { node, lane, edges, lanes: Math.max(before.length, next.length) };
  });
}

function CommitGraph({ row, width }: { row: GraphRowData; width: number }) {
  const colors = ["#1677ff", "#22a06b", "#8b5cf6", "#e67e22", "#e5484d", "#0891b2"];
  const x = (lane: number) => 14 + lane * 20;
  return <svg className="commit-graph" width={width} height="58" viewBox={`0 0 ${width} 58`} aria-hidden="true">{row.edges.map((edge, index) => {
    const startY = edge.half === "bottom" ? 29 : 0;
    const endY = edge.half === "top" ? 29 : 58;
    return <path key={`${index}-${edge.from}-${edge.to}`} d={`M ${x(edge.from)} ${startY} C ${x(edge.from)} ${(startY + endY) / 2}, ${x(edge.to)} ${(startY + endY) / 2}, ${x(edge.to)} ${endY}`} fill="none" stroke={colors[edge.to % colors.length]} strokeWidth="2" />;
  })}<circle cx={x(row.lane)} cy="29" r="5" fill="#fff" stroke={colors[row.lane % colors.length]} strokeWidth="3" /></svg>;
}

function BranchList({ branches, busy, onCheckout, onCreate, onDelete }: { branches: Branch[]; busy: boolean; onCheckout: (branch: string) => void; onCreate: () => void; onDelete: (branch: string) => void; }) {
  const [filter, setFilter] = useState("");
  const visible = branches.filter((branch) => branch.name.toLowerCase().includes(filter.toLowerCase()));
  return <section className="branches-workspace"><header><div><h3>本地分支</h3><p>切换、创建和安全删除本地分支。</p></div><div className="branch-tools"><label><Search /><input aria-label="筛选分支" value={filter} onChange={(event) => setFilter(event.target.value)} placeholder="筛选分支" /></label><Button type="primary" className="primary-button" icon={Plus} disabled={busy} onClick={onCreate}>新建分支</Button></div></header><div className="branch-list">{visible.map((branch) => <article key={branch.name}><Icon name="git" /><div><strong>{branch.name}</strong><p>{branch.commit_message || "没有提交说明"}</p><small>{branch.author} · {branch.commit_date} · {branch.commit_hash}</small></div>{branch.is_head ? <span>当前分支</span> : <button className="branch-action" disabled={busy} onClick={() => onCheckout(branch.name)}>切换</button>}{branch.is_protected && <span className="protected">受保护</span>}{branch.can_delete && <button className="icon-button danger" aria-label={`删除分支 ${branch.name}`} disabled={busy} title={`删除 ${branch.name}`} onClick={() => onDelete(branch.name)}><Trash2 /></button>}</article>)}</div></section>;
}

function Insights({ languages, stats, loading }: { languages: Language[] | null; stats: Stats | null; loading: boolean; }) {
  if (loading) return <Empty icon="chart" title="正在分析代码" text="首次分析会读取提交和受版本控制的文件。" />;
  if (!languages || !stats) return <Empty icon="chart" title="准备分析" text="正在准备数据…" />;
  const topLanguages = languages.slice(0, 5); const totalBytes = languages.reduce((sum, item) => sum + item.bytes, 0) || 1; const authors = [...stats.authors].sort((a, b) => b.lines_added - a.lines_added).slice(0, 8); const totalCode = authors.reduce((sum, item) => sum + item.lines_added, 0) || 1;
  return <section className="insights"><section className="insight-card"><div className="section-title"><div><h3>代码语言</h3><p>按当前受 Git 管理文件的大小计算。</p></div></div><div className="language-body"><Donut items={topLanguages.map((item) => ({ label: item.language, value: item.bytes }))} /><div className="legend">{topLanguages.map((item, index) => <div key={item.language}><i style={{ background: chartColors[index] }} /><span>{item.language}</span><strong>{Math.round(item.bytes / totalBytes * 100)}%</strong><small>{item.files} 个文件 · {formatBytes(item.bytes)}</small></div>)}</div></div></section><section className="insight-card"><div className="section-title"><div><h3>贡献者代码占比</h3><p>按历史提交中新增代码行估算，仅用于了解参与分布。</p></div></div><div className="bars">{authors.map((author) => { const percent = Math.round(author.lines_added / totalCode * 100); return <div className="bar-row" key={author.name}><div><strong>{author.name}</strong><span>{author.commits} 次提交 · {percent}%</span></div><div className="bar-track"><i style={{ width: `${percent}%` }} /></div><b>{author.lines_added.toLocaleString()} 行</b></div>; })}</div></section></section>;
}

function Donut({ items }: { items: { label: string; value: number }[] }) { const total = items.reduce((sum, item) => sum + item.value, 0) || 1; let offset = 0; return <figure className="donut" aria-label="代码语言占比图"><svg viewBox="0 0 42 42" role="img"><circle className="donut-base" cx="21" cy="21" r="15.9" />{items.map((item, index) => { const percentage = item.value / total * 100; const part = <circle key={item.label} cx="21" cy="21" r="15.9" fill="none" stroke={chartColors[index]} strokeWidth="5" strokeDasharray={`${percentage} ${100 - percentage}`} strokeDashoffset={-offset} />; offset += percentage; return part; })}</svg><figcaption><strong>{items.length}</strong><span>种语言</span></figcaption></figure>; }

function Bulk({ repos, operation, busy, onOperation, onRun }: { repos: string[]; operation: Operation; busy: boolean; onOperation: (operation: Operation) => void; onRun: () => void; }) { const labels: Record<Operation, [string, string]> = { status: ["检查状态", "只读取每个仓库的分支和工作区状态。"], pull: ["拉取更新", "从远端拉取已选仓库；有本地变更时 Git 会阻止危险覆盖。"], gc: ["清理 Git", "运行 git gc 回收本地 Git 对象，不删除工作区文件。"] }; return <section className="bulk-page"><div className="plain-intro"><h2>批量任务</h2><p>面向多个仓库的重复操作。先在仓库列表中明确勾选作用范围。</p></div><div className="bulk-grid"><section className="tool-card"><h3>选择操作</h3>{(Object.keys(labels) as Operation[]).map((item) => <button className={operation === item ? "operation selected" : "operation"} key={item} onClick={() => onOperation(item)}><strong>{labels[item][0]}</strong><span>{labels[item][1]}</span></button>)}<Button type="primary" className="primary-button" loading={busy} onClick={onRun} disabled={!repos.length}>{busy ? "正在执行…" : `对 ${repos.length} 个仓库执行`}</Button></section><aside className="scope-card"><h3>作用范围</h3>{repos.length ? repos.map((path) => <div key={path}><Icon name="folder" /><span>{nameOf(path)}</span></div>) : <p>还没有选择仓库。</p>}</aside></div></section>; }
function Activity({ runs }: { runs: Run[] }) { const labels: Record<Operation, string> = { status: "检查状态", pull: "拉取更新", gc: "清理 Git" }; return <section className="activity-page"><div className="plain-intro"><h2>操作记录</h2><p>记录当前会话内发起的批量任务及每个仓库的结果。</p></div>{runs.length ? runs.map((run) => <article className="run-card" key={run.id}><header><strong>{labels[run.operation]}</strong><small>{run.time} · {run.results.filter((item) => item.success).length}/{run.results.length} 成功</small></header>{run.results.map((result) => <div key={result.repo_path}><span className={result.success ? "ok" : "bad"}>{result.success ? "成功" : "失败"}</span><strong>{nameOf(result.repo_path)}</strong><small>{result.message}</small></div>)}</article>) : <Empty icon="clock" title="还没有操作记录" text="批量任务执行完成后，结果会显示在这里。" />}</section>; }

function Nav({ active, label, icon, onClick }: { active: boolean; label: string; icon: IconName; onClick: () => void; }) { return <button type="button" className={active ? "nav-item active" : "nav-item"} onClick={onClick}><Icon name={icon} />{label}</button>; }
function Tab({ active, label, onClick }: { active: boolean; label: string; onClick: () => void; }) { return <button type="button" role="tab" aria-selected={active} className={active ? "active" : ""} onClick={onClick}>{label}</button>; }
function Empty({ icon, title, text }: { icon: IconName; title: string; text: string; }) { return <section className="empty"><Icon name={icon} /><h3>{title}</h3><p>{text}</p></section>; }
const chartColors = ["#3b82f6", "#22c55e", "#a855f7", "#f59e0b", "#ef4444"];
type IconName = "folder" | "layers" | "clock" | "git" | "scan" | "arrow" | "arrowLeft" | "chart" | "check";
const iconMap: Record<IconName, LucideIcon> = { folder: Folder, layers: Layers3, clock: Clock3, git: GitBranch, scan: ScanSearch, arrow: ArrowRight, arrowLeft: ArrowLeft, chart: BarChart3, check: Check };
function Icon({ name }: { name: IconName }) { const Glyph = iconMap[name]; return <Glyph aria-hidden="true" className="icon" strokeWidth={1.8} />; }
