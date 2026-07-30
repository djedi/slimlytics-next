<script lang="ts">
  import AuthForm from '$lib/components/AuthForm.svelte';
</script>

<svelte:head>
  <title>Create account · Slimlytics</title>
  <meta
    name="description"
    content="Create a free Slimlytics account for privacy-first web analytics."
  />
</svelte:head>

<AuthForm mode="register" />
