<template>
  <UCard class="bg-gray-800 shadow-lg mb-8">
    <template #header>
      <h2 class="text-2xl font-semibold">Latest Updates</h2>
    </template>

    <div class="space-y-4">
      <UAccordion v-for="(update, index) in latestUpdates" :key="index" :items="[{
        label: `${update.date} - ${update.repo} - v${update.version}`,
        defaultOpen: false
      }]">
        <div class="p-4 bg-gray-700 rounded-md">
          <div class="prose prose-invert max-w-none" v-html="renderedChangelogs[index]"></div>
        </div>
      </UAccordion>
    </div>
  </UCard>
</template>

<script setup>
import { marked } from 'marked';

const latestUpdates = ref([]);
const renderedChangelogs = ref([]);

onMounted(async () => {
  try {
    const response = await fetch('/latest_updates');
    if (response.ok) {
      latestUpdates.value = await response.json();
      // Pré-rendre tous les changelogs avec marked
      renderedChangelogs.value = latestUpdates.value.map(update =>
        marked(update.changelog)
      );
    } else {
      console.error('Erreur lors de la récupération des mises à jour');
    }
  } catch (error) {
    console.error('Erreur:', error);
  }
});
</script>

<style>
/* Style pour le markdown rendu */
.prose h1, .prose h2, .prose h3 {
  margin-top: 1em;
  margin-bottom: 0.5em;
  font-weight: 600;
}

.prose ul {
  list-style-type: disc;
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.prose p {
  margin: 0.5em 0;
}

.prose a {
  color: #60a5fa;
  text-decoration: underline;
}

.prose code {
  background-color: rgba(0, 0, 0, 0.1);
  padding: 0.1em 0.3em;
  border-radius: 0.2em;
}

.prose blockquote {
  border-left: 4px solid #4b5563;
  padding-left: 1em;
  font-style: italic;
  margin: 0.5em 0;
}
</style>
