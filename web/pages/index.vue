<template>
  <div>
    <!-- Section des dernières mises à jour -->
    <LatestUpdates />

    <!-- Section des dépôts GitHub et Docker -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      <GithubRepoSection />
      <DockerRepoSection />
    </div>
  </div>
</template>
