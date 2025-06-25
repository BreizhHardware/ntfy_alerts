<template>
  <UCard class="bg-emerald-950 shadow-lg">
    <template #header>
      <h2 class="text-2xl font-semibold">Add a Github Repo</h2>
    </template>

    <form @submit.prevent="addRepo">
      <UFormGroup label="Name of the Github Repo" name="repo">
        <div class="flex items-center">
          <UBadge class="mr-2 py-2.5 px-3 bg-gray-700 text-gray-400">github.com/</UBadge>
          <UInput
            v-model="repoName"
            placeholder="BreizhHardware/ntfy_alerts"
            class="flex-1 bg-gray-700"
          />
        </div>
      </UFormGroup>
      <div class="flex justify-end gap-4 mt-4">
        <UButton color="gray" variant="ghost" @click="repoName = ''">Cancel</UButton>
        <UButton type="submit" color="green" variant="solid">Save</UButton>
      </div>
    </form>

    <template #footer>
      <div class="mt-4">
        <h3 class="text-lg font-semibold mb-2">Watched Github Repositories</h3>
        <UList v-if="watchedRepos.length" class="space-y-2">
          <UListItem v-for="repo in watchedRepos" :key="repo" class="flex justify-between items-center">
            <span>{{ repo }}</span>
            <UButton
              color="red"
              variant="ghost"
              icon="i-heroicons-x-mark"
              size="xs"
              @click="removeRepo(repo)"
            />
          </UListItem>
        </UList>
        <p v-else class="text-gray-400 italic">No repositories being watched</p>
      </div>
    </template>
  </UCard>
</template>

<script setup>
const repoName = ref('')
const watchedRepos = ref([])

onMounted(() => {
  refreshWatchedRepos()
})

async function addRepo() {
  if (!repoName.value) return

  try {
    const response = await fetch('/app_github_repo', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ repo: repoName.value })
    })

    if (response.ok) {
      repoName.value = ''
      await refreshWatchedRepos()
    } else {
      throw new Error('Failed to add repository')
    }
  } catch (error) {
    console.error('Error:', error)
  }
}

async function refreshWatchedRepos() {
  try {
    const response = await fetch('/watched_repos')
    if (response.ok) {
      watchedRepos.value = await response.json()
    }
  } catch (error) {
    console.error('Error fetching watched repos:', error)
  }
}

async function removeRepo(repo) {
  try {
    const response = await fetch('/delete_repo', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ repo })
    })

    if (response.ok) {
      await refreshWatchedRepos()
    } else {
      throw new Error('Failed to remove repository')
    }
  } catch (error) {
    console.error('Error:', error)
  }
}
</script>
