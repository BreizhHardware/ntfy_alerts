<template>
  <UCard class="bg-gray-800 shadow-lg mb-8">
    <template #header>
      <h2 class="text-2xl font-semibold">Dernières mises à jour</h2>
    </template>

    <div class="space-y-4">
      <UAccordion v-for="(update, index) in latestUpdates" :key="index" :items="[{
        label: `${update.date} - ${update.repo} - v${update.version}`,
        defaultOpen: index === 0
      }]">
        <div class="p-4 bg-gray-700 rounded-md">
          <div v-html="formatChangelog(update.changelog)"></div>
        </div>
      </UAccordion>
    </div>
  </UCard>
</template>

<script setup>
const latestUpdates = ref([]);

onMounted(async () => {
  try {
    const response = await fetch('/latest_updates');
    if (response.ok) {
      latestUpdates.value = await response.json();
    } else {
      console.error('Erreur lors de la récupération des mises à jour');
    }
  } catch (error) {
    console.error('Erreur:', error);
  }
});

function formatChangelog(changelog) {
  return changelog
    .trim()
    .split('\n')
    .map(line => {
      if (line.startsWith('-')) {
        return `<li class="ml-4 list-disc">${line.substring(1).trim()}</li>`;
      }
      return line;
    })
    .join('\n');
}
</script>
