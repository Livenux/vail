<script lang="ts">
  import { onMount } from 'svelte';
  import Login from './routes/Login.svelte';
  import Dashboard from './routes/Dashboard.svelte';
  import { auth, checkAuth } from './lib/auth';

  let ready = $state(false);

  onMount(() => {
    checkAuth();
    ready = true;
  });
</script>

{#if ready}
  {#if $auth.isAuthenticated}
    <Dashboard />
  {:else}
    <Login />
  {/if}
{/if}
