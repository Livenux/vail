<script lang="ts">
  import { auth } from '../lib/auth';
  import { hostApi, sftpApi, type Host, type UploadTask } from '../lib/api';
  import { onMount } from 'svelte';

  let hosts = $state<Host[]>([]);
  let tasks = $state<UploadTask[]>([]);
  let loading = $state(true);
  let activeTab = $state<'hosts' | 'upload'>('hosts');

  onMount(async () => {
    try {
      const [hostsRes, tasksRes] = await Promise.all([
        hostApi.list(),
        sftpApi.listTasks()
      ]);
      if (hostsRes.data.code === 200) hosts = hostsRes.data.data || [];
      if (tasksRes.data.code === 200) tasks = tasksRes.data.data || [];
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  });

  async function handleLogout() {
    await auth.logout();
  }
</script>

<div class="layout">
  <aside class="sidebar">
    <div class="brand">
      <h2>Vail</h2>
    </div>
    <nav>
      <button class="nav-item" class:active={activeTab === 'hosts'} onclick={() => activeTab = 'hosts'}>
        主机管理
      </button>
      <button class="nav-item" class:active={activeTab === 'upload'} onclick={() => activeTab = 'upload'}>
        文件上传
      </button>
    </nav>
    <div class="user-info">
      <span>{$auth.user?.nickname || $auth.user?.username}</span>
      <button onclick={handleLogout}>退出</button>
    </div>
  </aside>

  <main class="main-content">
    <header class="header">
      <h1>{activeTab === 'hosts' ? '主机管理' : '文件上传'}</h1>
    </header>

    {#if loading}
      <div class="loading">加载中...</div>
    {:else if activeTab === 'hosts'}
      <div class="card">
        <div class="table-header">
          <h3>主机列表</h3>
        </div>
        {#if hosts.length === 0}
          <p class="text-secondary text-center mt-4">暂无主机</p>
        {:else}
          <table>
            <thead>
              <tr>
                <th>名称</th>
                <th>主机</th>
                <th>端口</th>
                <th>用户名</th>
                <th>状态</th>
              </tr>
            </thead>
            <tbody>
              {#each hosts as host}
                <tr>
                  <td>{host.name}</td>
                  <td>{host.hostname}</td>
                  <td>{host.port}</td>
                  <td>{host.username || '-'}</td>
                  <td>
                    <span class="status-badge" class:active={host.status === 1}>
                      {host.status === 1 ? '正常' : '禁用'}
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {:else}
      <div class="card">
        <div class="table-header">
          <h3>上传任务</h3>
        </div>
        {#if tasks.length === 0}
          <p class="text-secondary text-center mt-4">暂无上传任务</p>
        {:else}
          <table>
            <thead>
              <tr>
                <th>任务号</th>
                <th>文件大小</th>
                <th>已上传</th>
                <th>状态</th>
                <th>创建时间</th>
              </tr>
            </thead>
            <tbody>
              {#each tasks as task}
                <tr>
                  <td>{task.task_no.slice(0, 8)}...</td>
                  <td>{(task.file_size / 1024 / 1024).toFixed(2)} MB</td>
                  <td>{(task.uploaded_size / 1024 / 1024).toFixed(2)} MB</td>
                  <td>
                    <span class="status-badge" class:active={task.status === 2} class:pending={task.status === 0}>
                      {task.status === 0 ? '等待中' : task.status === 1 ? '上传中' : '已完成'}
                    </span>
                  </td>
                  <td>{new Date(task.create_time).toLocaleString()}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  .layout {
    display: flex;
    min-height: 100vh;
  }

  .sidebar {
    width: 220px;
    background: #1d2129;
    color: white;
    display: flex;
    flex-direction: column;
  }

  .brand {
    padding: 20px;
    border-bottom: 1px solid #2d333b;
  }

  .brand h2 {
    color: var(--primary);
    margin: 0;
  }

  nav {
    flex: 1;
    padding: 12px;
  }

  .nav-item {
    width: 100%;
    text-align: left;
    background: transparent;
    color: #adbac7;
    padding: 10px 16px;
    border-radius: 6px;
    margin-bottom: 4px;
  }

  .nav-item:hover {
    background: #2d333b;
  }

  .nav-item.active {
    background: var(--primary);
    color: white;
  }

  .user-info {
    padding: 16px;
    border-top: 1px solid #2d333b;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .user-info button {
    padding: 4px 12px;
    font-size: 12px;
    background: #373e47;
  }

  .main-content {
    flex: 1;
    background: var(--bg);
  }

  .header {
    background: white;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border);
  }

  .header h1 {
    font-size: 20px;
    margin: 0;
  }

  .container {
    padding: 24px;
  }

  .table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .table-header h3 {
    margin: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .status-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    background: #e5e6eb;
  }

  .status-badge.active {
    background: #d9f7be;
    color: #389e0d;
  }

  .status-badge.pending {
    background: #fff1b8;
    color: #d4b106;
  }

  .loading {
    text-align: center;
    padding: 40px;
    color: var(--text-secondary);
  }
</style>
