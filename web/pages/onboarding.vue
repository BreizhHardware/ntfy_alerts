<template>
  <div class="min-h-screen bg-gray-900 p-6">
    <div class="max-w-3xl mx-auto bg-gray-800 rounded-lg shadow-lg overflow-hidden">
      <div class="p-6 border-b border-gray-700">
        <h1 class="text-2xl font-bold text-white">Application Setup</h1>
        <p class="mt-2 text-gray-400">Configure your application and create an administrator account</p>
      </div>

      <UStepper v-model="step" :items="steps" class="p-6">
        <template #item="{ item }">
          <h2 class="text-lg font-medium">{{ item.title }}</h2>
          <p class="text-sm text-gray-400">{{ item.description }}</p>
        </template>
      </UStepper>

      <div class="p-6">
        <!-- Step 1: Create Administrator Account -->
        <div v-if="step === 1" class="space-y-6">
          <div>
            <h3 class="text-lg font-medium text-white mb-4">Create Administrator Account</h3>
            <p class="text-sm text-gray-400 mb-6">This account will have full access to manage the application</p>

            <div class="space-y-4">
              <div>
                <label for="username" class="block text-sm font-medium text-gray-400">Username</label>
                <UInput
                  id="username"
                  v-model="adminUser.username"
                  placeholder="admin"
                  class="w-full"
                />
              </div>

              <div>
                <label for="password" class="block text-sm font-medium text-gray-400">Password</label>
                <UInput
                  id="password"
                  v-model="adminUser.password"
                  type="password"
                  placeholder="********"
                  class="w-full"
                />
              </div>

              <div>
                <label for="confirmPassword" class="block text-sm font-medium text-gray-400">Confirm Password</label>
                <UInput
                  id="confirmPassword"
                  v-model="adminUser.confirmPassword"
                  type="password"
                  placeholder="********"
                  class="w-full"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Main notification service -->
        <div v-if="step === 2" class="space-y-6">
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-2">Main notification service</label>
            <USelect
              v-model="selectedService"
              :options="notificationServices"
              option-attribute="label"
              value-attribute="value"
              placeholder="Select a notification service"
              class="w-full"
            />
          </div>

          <!-- NTFY Configuration -->
          <div v-if="selectedService === 'ntfy'" class="space-y-4">
            <div>
              <label for="ntfy_url" class="block text-sm font-medium text-gray-400">NTFY URL</label>
              <UInput
                id="ntfy_url"
                v-model="settings.ntfy_url"
                placeholder="https://ntfy.sh/your-topic"
                class="w-full"
              />
            </div>
          </div>

          <!-- Discord Configuration -->
          <div v-if="selectedService === 'discord'" class="space-y-4">
            <div>
              <label for="discord_webhook" class="block text-sm font-medium text-gray-400">Discord Webhook URL</label>
              <UInput
                id="discord_webhook"
                v-model="settings.discord_webhook_url"
                placeholder="https://discord.com/api/webhooks/..."
                class="w-full"
              />
            </div>
          </div>

          <!-- Slack Configuration -->
          <div v-if="selectedService === 'slack'" class="space-y-4">
            <div>
              <label for="slack_webhook" class="block text-sm font-medium text-gray-400">Slack Webhook URL</label>
              <UInput
                id="slack_webhook"
                v-model="settings.slack_webhook_url"
                placeholder="https://hooks.slack.com/services/..."
                class="w-full"
              />
            </div>
          </div>

          <!-- Gotify Configuration -->
          <div v-if="selectedService === 'gotify'" class="space-y-4">
            <div>
              <label for="gotify_url" class="block text-sm font-medium text-gray-400">Gotify URL</label>
              <UInput
                id="gotify_url"
                v-model="settings.gotify_url"
                placeholder="https://gotify.example.com"
                class="w-full"
              />
            </div>
            <div>
              <label for="gotify_token" class="block text-sm font-medium text-gray-400">Gotify Token</label>
              <UInput
                id="gotify_token"
                v-model="settings.gotify_token"
                placeholder="Axxxxxxxxx.xxxxx"
                class="w-full"
              />
            </div>
          </div>
        </div>

        <!-- Step 3: GitHub Settings -->
        <div v-if="step === 3" class="space-y-6">
          <div>
            <label for="github_token" class="block text-sm font-medium text-gray-400">GitHub Token (optional)</label>
            <UInput
              id="github_token"
              v-model="settings.github_token"
              placeholder="ghp_xxxxxxxxxxxxxxxx"
              class="w-full"
            />
            <p class="mt-1 text-xs text-gray-500">
              A GitHub token helps avoid API rate limits for private repositories
            </p>
          </div>
        </div>

        <!-- Step 4: Docker Hub Settings -->
        <div v-if="step === 4" class="space-y-6">
          <div>
            <label for="docker_username" class="block text-sm font-medium text-gray-400">Docker Hub Username (optional)</label>
            <UInput
              id="docker_username"
              v-model="settings.docker_username"
              placeholder="username"
              class="w-full"
            />
          </div>
          <div>
            <label for="docker_password" class="block text-sm font-medium text-gray-400">Docker Hub Password (optionnel)</label>
            <UInput
              id="docker_password"
              v-model="settings.docker_password"
              type="password"
              placeholder="********"
              class="w-full"
            />
            <p class="mt-1 text-xs text-gray-500">
              Docker Hub credentials allow access to private images
            </p>
          </div>
        </div>

        <!-- Step 5: Advanced Settings -->
        <div v-if="step === 5" class="space-y-6">
          <div>
            <label for="check_interval" class="block text-sm font-medium text-gray-400">Check Interval (seconds)</label>
            <UInput
              id="check_interval"
              v-model="settings.check_interval"
              type="number"
              min="60"
              placeholder="3600"
              class="w-full"
            />
            <p class="mt-1 text-xs text-gray-500">
              Default interval is 3600 seconds (1 hour)
            </p>
          </div>
        </div>

        <div v-if="error" class="mt-6 p-3 text-sm text-red-500 bg-red-100 rounded-md">
          {{ error }}
        </div>

        <div class="flex justify-between mt-8">
          <UButton
            v-if="step > 1"
            @click="step--"
            color="gray"
          >
            Previous
          </UButton>
          <div v-else></div>

          <UButton
            v-if="step < steps.length"
            @click="nextStep"
            color="primary"
          >
            Next
          </UButton>
          <UButton
            v-else
            @click="saveSettings"
            color="primary"
            :loading="loading"
          >
            Complete Setup
          </UButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
const auth = useAuth();
const router = useRouter();
const route = useRoute();

// Check if admin exists and redirect accordingly
onMounted(async () => {
  try {
    // Check if force parameter is present in the URL
    const forceOnboarding = route.query.force === 'true';

    // If forcing onboarding and user is authenticated as admin, allow access
    if (forceOnboarding && auth.isAuthenticated.value && auth.isAdmin.value) {
      // Load existing settings if available
      await loadExistingSettings();
      return;
    }

    // Otherwise check if admin exists
    const response = await fetch('/is_configured');
    if (response.ok) {
      const data = await response.json();
      const adminExists = data.data && data.data.admin_exists;

      // If admin exists, redirect to login or dashboard
      if (adminExists) {
        if (auth.isAuthenticated.value) {
          router.push('/');
        } else {
          router.push('/login');
        }
      }
    }
  } catch (err) {
    console.error('Error checking configuration:', err);
  }
});

// Admin user creation data
const adminUser = reactive({
  username: '',
  password: '',
  confirmPassword: '',
});

// Onboarding steps
const steps = [
  { title: 'Create Admin', description: 'Create your administrator account' },
  { title: 'Notification Service', description: 'Choose your main notification service' },
  { title: 'GitHub Settings', description: 'Configure options for GitHub' },
  { title: 'Docker Hub Settings', description: 'Configure options for Docker Hub' },
  { title: 'Advanced Settings', description: 'Configure additional options' }
];

const step = ref(1);
const selectedService = ref(null);
const error = ref('');
const loading = ref(false);

// List of available notification services
const notificationServices = [
  { label: 'NTFY', value: 'ntfy' },
  { label: 'Discord', value: 'discord' },
  { label: 'Slack', value: 'slack' },
  { label: 'Gotify', value: 'gotify' }
];

// Application settings
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

// Function to proceed to next step
async function nextStep() {
  // Validate current step
  if (step.value === 1) {
    // Validate admin user creation
    if (!adminUser.username) {
      error.value = 'Please enter a username';
      return;
    }
    if (!adminUser.password) {
      error.value = 'Please enter a password';
      return;
    }
    if (adminUser.password !== adminUser.confirmPassword) {
      error.value = 'Passwords do not match';
      return;
    }

    // Create admin user
    try {
      error.value = '';
      loading.value = true;

      // Register admin user
      await auth.register(adminUser.username, adminUser.password, true);

      // Continue to next step
      loading.value = false;
      step.value++;
      return;
    } catch (err) {
      error.value = err.message || 'Error creating admin user';
      loading.value = false;
      return;
    }
  }
  else if (step.value === 2) {
    if (!selectedService.value) {
      error.value = 'Please select a notification service';
      return;
    }

    // Validate selected service
    if (selectedService.value === 'ntfy' && !settings.ntfy_url) {
      error.value = 'Please enter the NTFY URL';
      return;
    } else if (selectedService.value === 'discord' && !settings.discord_webhook_url) {
      error.value = 'Please enter the Discord webhook URL';
      return;
    } else if (selectedService.value === 'slack' && !settings.slack_webhook_url) {
      error.value = 'Please enter the Slack webhook URL';
      return;
    } else if (selectedService.value === 'gotify' && (!settings.gotify_url || !settings.gotify_token)) {
      error.value = 'Please enter both Gotify URL and token';
      return;
    }
  }

  // Reset error and proceed to next step
  error.value = '';
  step.value++;
}

// Function to save settings
async function saveSettings() {
  try {
    loading.value = true;

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

    // Redirect to main page
    router.push('/');
  } catch (err) {
    error.value = err.message || 'An error occurred while saving settings';
  } finally {
    loading.value = false;
  }
}

// Function to load existing settings
async function loadExistingSettings() {
  try {
    if (!auth.isAuthenticated.value) return;

    const response = await fetch('/settings', {
      headers: {
        'Authorization': auth.token.value
      }
    });

    if (response.ok) {
      const data = await response.json();
      if (data.success && data.data) {
        // Populate settings with existing values
        const existingSettings = data.data;

        // Update notification service selection
        if (existingSettings.ntfy_url) {
          selectedService.value = 'ntfy';
          settings.ntfy_url = existingSettings.ntfy_url;
        } else if (existingSettings.discord_webhook_url) {
          selectedService.value = 'discord';
          settings.discord_webhook_url = existingSettings.discord_webhook_url;
        } else if (existingSettings.slack_webhook_url) {
          selectedService.value = 'slack';
          settings.slack_webhook_url = existingSettings.slack_webhook_url;
        } else if (existingSettings.gotify_url) {
          selectedService.value = 'gotify';
          settings.gotify_url = existingSettings.gotify_url;
          settings.gotify_token = existingSettings.gotify_token;
        }

        // Update other settings
        settings.github_token = existingSettings.github_token || '';
        settings.docker_username = existingSettings.docker_username || '';
        settings.docker_password = existingSettings.docker_password || '';
        settings.check_interval = existingSettings.check_interval || 3600;
      }
    }
  } catch (err) {
    console.error('Error loading existing settings:', err);
  }
}
</script>
