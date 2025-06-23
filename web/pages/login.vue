<template>
  <div class="flex items-center justify-center min-h-screen bg-gray-900">
    <div class="w-full max-w-md p-8 space-y-8 bg-gray-800 rounded-lg shadow-lg">
      <div class="text-center">
        <h1 class="text-2xl font-bold text-white">Login</h1>
        <p class="mt-2 text-sm text-gray-400">Sign in to manage your notifications</p>
      </div>

      <form @submit.prevent="handleLogin" class="mt-8 space-y-6">
        <div>
          <label for="username" class="block text-sm font-medium text-gray-400">Username</label>
          <input
            id="username"
            v-model="form.username"
            type="text"
            required
            class="block w-full px-3 py-2 mt-1 text-gray-900 placeholder-gray-500 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-gray-400">Password</label>
          <input
            id="password"
            v-model="form.password"
            type="password"
            required
            class="block w-full px-3 py-2 mt-1 text-gray-900 placeholder-gray-500 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>

        <div v-if="error" class="p-3 text-sm text-red-500 bg-red-100 rounded-md">
          {{ error }}
        </div>

        <div>
          <UButton
            type="submit"
            color="primary"
            block
            :loading="loading"
          >
            Login
          </UButton>
        </div>
      </form>

      <div class="text-center mt-4">
        <p class="text-sm text-gray-400">
          First time?
          <NuxtLink to="/onboarding" class="font-medium text-indigo-400 hover:text-indigo-300">
            Setup your application
          </NuxtLink>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
const auth = useAuth();
const router = useRouter();

const form = reactive({
  username: '',
  password: ''
});

const error = ref('');
const loading = ref(false);

async function handleLogin() {
  try {
    loading.value = true;
    error.value = '';

    await auth.login(form.username, form.password);

    // Redirect to main page or configuration page if needed
    if (auth.isFirstLogin.value) {
      router.push('/onboarding');
    } else {
      router.push('/');
    }
  } catch (err) {
    error.value = err.message || 'An error occurred during login';
  } finally {
    loading.value = false;
  }
}
</script>
