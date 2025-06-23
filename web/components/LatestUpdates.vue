<template>
  <UCard class="bg-gray-800 shadow-lg mb-8">
    <template #header>
      <h2 class="text-2xl font-semibold">Latest Updates</h2>
    </template>

    <div class="space-y-4">
      <div v-for="(update, index) in latestUpdates" :key="index" class="border border-gray-700 rounded-md overflow-hidden">
        <button
          @click="toggleChangelog(index)"
          class="w-full flex justify-between items-center px-4 py-3 bg-gray-700 hover:bg-gray-600 transition-colors text-left"
        >
          <div>
            <span class="font-medium">{{ update.repo }} - v{{ update.version }}</span>
            <div class="text-sm text-gray-400">{{ update.date }}</div>
          </div>
          <UIcon :name="openStates[index] ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'" class="text-gray-400" />
        </button>
        <div
          v-show="openStates[index]"
          class="p-4 bg-gray-800 prose prose-invert max-w-none transition-all"
          v-html="renderedChangelogs[index]"
        ></div>
      </div>
    </div>
  </UCard>
</template>

<script setup>
import { marked } from 'marked';

const latestUpdates = ref([]);
const renderedChangelogs = ref([]);
const openStates = ref([]);

onMounted(async () => {
  try {
    const response = await fetch('/latest_updates');
    if (response.ok) {
      latestUpdates.value = await response.json();
      renderedChangelogs.value = latestUpdates.value.map(update =>
        marked(update.changelog)
      );
      openStates.value = Array(latestUpdates.value.length).fill(false);
    } else {
      console.error('Erreur lors de la récupération des mises à jour');
    }
  } catch (error) {
    console.error('Erreur:', error);
  }
});

function toggleChangelog(index) {
  openStates.value[index] = !openStates.value[index];
}
</script>

<style>
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
