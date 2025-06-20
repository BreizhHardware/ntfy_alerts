<template>
  <UCard class="bg-emerald-950 shadow-lg">
    <template #header>
      <h2 class="text-2xl font-semibold">Add a Docker Repo</h2>
    </template>

    <form @submit.prevent="addDockerRepo">
      <UFormGroup label="Name of the Docker Repo" name="dockerRepo">
        <div class="flex items-center">
          <UBadge class="mr-2 py-2.5 px-3 bg-gray-700 text-gray-400">hub.docker.com/r/</UBadge>
          <UInput
            v-model="dockerRepoName"
            placeholder="breizhhardware/github-ntfy"
            class="flex-1 bg-gray-700"
          />
        </div>
      </UFormGroup>
      <div class="flex justify-end gap-4 mt-4">
        <UButton color="gray" variant="ghost" @click="dockerRepoName = ''">Cancel</UButton>
        <UButton type="submit" color="green" variant="solid">Save</UButton>
      </div>
    </form>

    <template #footer>
      <div class="mt-4">
        <h3 class="text-lg font-semibold mb-2">Watched Docker Repositories</h3>
        <UList v-if="watchedDockerRepos.length" class="space-y-2">
          <UListItem v-for="repo in watchedDockerRepos" :key="repo" class="flex justify-between items-center">
            <span>{{ repo }}</span>
            <UButton
              color="red"
              variant="ghost"
              icon="i-heroicons-x-mark"
              size="xs"
              @click="removeDockerRepo(repo)"
            />
          </UListItem>
        </UList>
        <p v-else class="text-gray-400 italic">No Docker repositories being watched</p>
      </div>
    </template>
  </UCard>
</template>

<script setup>
const dockerRepoName = ref('')
const watchedDockerRepos = ref([])

onMounted(() => {
  refreshWatchedDockerRepos()
})

async function addDockerRepo() {
  if (!dockerRepoName.value) return

  try {
    const response = await fetch('/app_docker_repo', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ repo: dockerRepoName.value })
    })

    if (response.ok) {
      dockerRepoName.value = ''
      await refreshWatchedDockerRepos()
    } else {
      throw new Error('Failed to add Docker repository')
    }
  } catch (error) {
    console.error('Error:', error)
  }
}

async function refreshWatchedDockerRepos() {
  try {
    const response = await fetch('/watched_docker_repos')
    if (response.ok) {
      watchedDockerRepos.value = await response.json()
    }
  } catch (error) {
    console.error('Error fetching watched Docker repos:', error)
  }
}

async function removeDockerRepo(repo) {
  try {
    const response = await fetch('/delete_docker_repo', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ repo })
    })

    if (response.ok) {
      await refreshWatchedDockerRepos()
    } else {
      throw new Error('Failed to remove Docker repository')
    }
  } catch (error) {
    console.error('Error:', error)
  }
}
</script>
