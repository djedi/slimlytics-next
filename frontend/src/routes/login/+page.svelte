<script lang="ts">
  import AuthForm from '$lib/components/AuthForm.svelte';
</script>

<svelte:head>
  <title>Sign in · Slimlytics</title>
  <meta name="description" content="Sign in to your Slimlytics analytics workspace." />
</svelte:head>

<AuthForm mode="login" />
