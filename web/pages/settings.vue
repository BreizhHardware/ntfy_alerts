<template>
  <div>
    <AppHeader />

    <div class="container mx-auto px-4 py-8">
      <h1 class="text-2xl font-bold text-white mb-8">Settings</h1>

      <UCard class="mb-8">
        <template #header>
          <div class="flex justify-between items-center">
            <h2 class="text-xl font-semibold">Notification Services</h2>
          </div>
        </template>

        <div class="space-y-6">
          <!-- NTFY -->
          <div>
            <h3 class="text-lg font-medium mb-2">NTFY</h3>
            <UInput
              v-model="settings.ntfy_url"
              label="NTFY URL"
              placeholder="https://ntfy.sh/your-topic"
              class="w-full"
            />
          </div>

          <!-- Discord -->
          <div>
            <h3 class="text-lg font-medium mb-2">Discord</h3>
            <UInput
              v-model="settings.discord_webhook_url"
              label="Discord Webhook URL"
              placeholder="https://discord.com/api/webhooks/..."
              class="w-full"
            />
          </div>

          <!-- Slack -->
          <div>
            <h3 class="text-lg font-medium mb-2">Slack</h3>
            <UInput
              v-model="settings.slack_webhook_url"
              label="Slack Webhook URL"
              placeholder="https://hooks.slack.com/services/..."
              class="w-full"
            />
          </div>

          <!-- Gotify -->
          <div>
            <h3 class="text-lg font-medium mb-2">Gotify</h3>
            <div class="space-y-2">
              <UInput
                v-model="settings.gotify_url"
                label="Gotify URL"
                placeholder="https://gotify.example.com"
                class="w-full"
              />
              <UInput
                v-model="settings.gotify_token"
                label="Gotify Token"
                placeholder="Axxxxxxxxx.xxxxx"
                class="w-full"
              />
            </div>
          </div>
        </div>
      </UCard>

      <UCard class="mb-8">
        <template #header>
          <div class="flex justify-between items-center">
            <h2 class="text-xl font-semibold">GitHub</h2>
          </div>
        </template>

        <div>
          <UInput
            v-model="settings.github_token"
            label="GitHub Token (optional)"
            placeholder="ghp_xxxxxxxxxxxxxxxx"
            class="w-full"
          />
          <p class="mt-1 text-xs text-gray-500">
            A GitHub token helps avoid API rate limits for private repositories
          </p>
        </div>
      </UCard>

      <UCard class="mb-8">
        <template #header>
          <div class="flex justify-between items-center">
            <h2 class="text-xl font-semibold">Docker Hub</h2>
          </div>
        </template>

        <div class="space-y-4">
          <UInput
            v-model="settings.docker_username"
            label="Docker Hub Username (optional)"
            placeholder="username"
            class="w-full"
          />
          <UInput
            v-model="settings.docker_password"
            label="Docker Hub Password (optional)"
            type="password"
            placeholder="********"
            class="w-full"
          />
          <p class="mt-1 text-xs text-gray-500">
            Docker Hub credentials allow access to private images
          </p>
        </div>
      </UCard>

      <UCard class="mb-8">
        <template #header>
          <div class="flex justify-between items-center">
            <h2 class="text-xl font-semibold">Advanced Settings</h2>
          </div>
        </template>

        <div>
          <UInput
            v-model="settings.check_interval"
            label="Check Interval (seconds)"
            type="number"
            min="60"
            placeholder="3600"
            class="w-full"
          />
          <p class="mt-1 text-xs text-gray-500">
            Default interval is 3600 seconds (1 hour)
          </p>
        </div>
      </UCard>

      <div v-if="error" class="p-3 mb-6 text-sm text-red-500 bg-red-100 rounded-md">
        {{ error }}
      </div>

      <div v-if="success" class="p-3 mb-6 text-sm text-green-500 bg-green-100 rounded-md">
        {{ success }}
      </div>

      <div class="flex justify-end">
        <UButton
          @click="saveSettings"
          color="primary"
          :loading="loading"
        >
          Save Changes
        </UButton>
      </div>
    </div>

    <AppFooter />
  </div>
</template>

<script setup>
const auth = useAuth();
const router = useRouter();

// Check if user is authenticated
onMounted(async () => {
  if (!auth.isAuthenticated.value) {
    return router.push('/login');
  }

  // Load current settings
  await loadSettings();
});

const settings = reactive({
  ntfy_url: '',
  github_token: '',
  docker_username: '',
  docker_password: '',
  gotify_url: '',
  gotify_token: '',
  discord_webhook_url: '',
  slack_webhook_url: '',
  check_interval: 3600
});

const error = ref('');
const success = ref('');
const loading = ref(false);

// Load current settings
async function loadSettings() {
  try {
    loading.value = true;

    const response = await fetch('/settings', {
      method: 'GET',
      headers: {
        'Authorization': auth.token.value
      }
    });

    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.message || 'Error loading settings');
    }

    const data = await response.json();

    if (data.success && data.data) {
      // Update settings with loaded values
      Object.assign(settings, data.data);
    }
  } catch (err) {
    error.value = err.message || 'An error occurred while loading settings';
  } finally {
    loading.value = false;
  }
}

// Function to save settings
async function saveSettings() {
  try {
    loading.value = true;
    error.value = '';
    success.value = '';

    // Prepare settings
    const now = new Date().toISOString();
    const settingsData = {
      ...settings,
      last_updated: now
    };

    // Send settings to server
    const response = await fetch('/settings', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': auth.token.value
      },
      body: JSON.stringify(settingsData)
    });

    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.message || 'Error saving settings');
    }

    success.value = 'Settings updated successfully';
  } catch (err) {
    error.value = err.message || 'An error occurred while saving settings';
  } finally {
    loading.value = false;
  }
}
</script>

