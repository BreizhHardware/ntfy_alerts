<template>
  <div class="min-h-screen bg-gray-900 text-gray-200">
    <UContainer>
      <AppHeader />
      <main class="py-8">
        <!-- Section des dernières mises à jour -->
        <LatestUpdates />

        <!-- Section des dépôts GitHub et Docker -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
          <GithubRepoSection />
          <DockerRepoSection />
        </div>
      </main>
      <AppFooter />
    </UContainer>
  </div>
</template>

<script setup>
// No script content provided in the original code or the change description
</script>

<style>
/* No style content provided in the original code or the change description */
</style>
