<script lang="ts">
  import { auth } from '../lib/auth';
  import { authApi } from '../lib/api';

  let username = $state('');
  let password = $state('');
  let loading = $state(false);
  let error = $state('');

  async function handleLogin() {
    if (!username || !password) {
      error = '请输入用户名和密码';
      return;
    }

    loading = true;
    error = '';

    try {
      const res = await authApi.login({ username, password });
      if (res.data.code === 200 && res.data.data) {
        const { access_token, refresh_token, user } = res.data.data;
        auth.login(access_token, refresh_token, user);
      } else {
        error = res.data.message || '登录失败';
      }
    } catch (e: any) {
      error = e.response?.data?.message || '登录失败，请稍后重试';
    } finally {
      loading = false;
    }
  }
</script>

<div class="login-page">
  <div class="login-card">
    <div class="logo">
      <h1>Vail</h1>
      <p class="text-secondary">运维堡垒机</p>
    </div>

    <form onsubmit={(e) => { e.preventDefault(); handleLogin(); }}>
      <div class="form-group">
        <label for="username">用户名</label>
        <input 
          id="username" 
          type="text" 
          bind:value={username} 
          placeholder="请输入用户名"
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="password">密码</label>
        <input 
          id="password" 
          type="password" 
          bind:value={password} 
          placeholder="请输入密码"
          disabled={loading}
        />
      </div>

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <button type="submit" disabled={loading}>
        {loading ? '登录中...' : '登录'}
      </button>
    </form>
  </div>
</div>

<style>
  .login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .login-card {
    background: white;
    padding: 40px;
    border-radius: 12px;
    width: 400px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  }

  .logo {
    text-align: center;
    margin-bottom: 32px;
  }

  .logo h1 {
    font-size: 32px;
    color: var(--primary);
    margin-bottom: 8px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-weight: 500;
  }

  .error {
    color: var(--danger);
    font-size: 14px;
    margin-bottom: 16px;
    text-align: center;
  }

  button {
    width: 100%;
    padding: 12px;
    font-size: 16px;
  }
</style>
