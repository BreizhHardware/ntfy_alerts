<template>
  <div class="flex items-center justify-center min-h-screen bg-gray-900">
    <div class="w-full max-w-md p-8 space-y-8 bg-gray-800 rounded-lg shadow-lg">
      <div class="text-center">
        <h1 class="text-2xl font-bold text-white">Create an account</h1>
        <p class="mt-2 text-sm text-gray-400">Sign up to manage your notifications</p>
      </div>

      <div v-if="existingUsers && !isAdminUser" class="p-4 mb-4 text-sm text-yellow-400 bg-yellow-900 bg-opacity-30 rounded-md">
        New accounts require administrator approval. Your registration will be submitted for review.
      </div>

      <form @submit.prevent="handleRegister" class="mt-8 space-y-6">
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

        <div>
          <label for="confirmPassword" class="block text-sm font-medium text-gray-400">Confirm password</label>
          <input
            id="confirmPassword"
            v-model="form.confirmPassword"
            type="password"
            required
            class="block w-full px-3 py-2 mt-1 text-gray-900 placeholder-gray-500 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>

        <div v-if="!existingUsers">
          <div class="flex items-center">
            <input
              id="isAdmin"
              v-model="form.isAdmin"
              type="checkbox"
              class="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
            />
            <label for="isAdmin" class="block ml-2 text-sm font-medium text-gray-400">
              Admin account
            </label>
          </div>
          <p class="mt-1 text-xs text-gray-500">
            Check this box for the first administrator account
          </p>
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
            {{ existingUsers && !isAdminUser ? 'Submit Registration Request' : 'Register' }}
          </UButton>
        </div>
      </form>

      <div class="text-center mt-4">
        <p class="text-sm text-gray-400">
          Already have an account?
          <NuxtLink to="/login" class="font-medium text-indigo-400 hover:text-indigo-300">
            Login
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
  password: '',
  confirmPassword: '',
  isAdmin: false
});

const error = ref('');
const loading = ref(false);
const existingUsers = ref(false);
const isAdminUser = ref(false);

// Check if users already exist and if current user is admin
onMounted(async () => {
  try {
    // Check if admin exists
    const response = await fetch('/is_configured');
    if (response.ok) {
      const data = await response.json();
      existingUsers.value = data.data && data.data.admin_exists;
    }

    // Check if current user is admin (if logged in)
    const userData = localStorage.getItem('user');
    if (userData) {
      const user = JSON.parse(userData);
      isAdminUser.value = user.is_admin === true;
    }
  } catch (err) {
    console.error('Error checking for existing users:', err);
  }
});

async function handleRegister() {
  try {
    // Check that passwords match
    if (form.password !== form.confirmPassword) {
      error.value = 'Passwords do not match';
      return;
    }

    loading.value = true;
    error.value = '';

    // If users exist and current user is not admin, set pending flag
    const isPending = existingUsers.value && !isAdminUser.value;

    await auth.register(form.username, form.password, existingUsers.value ? false : form.isAdmin, isPending);

    // Redirect to onboarding page if it's the first user
    if (!existingUsers.value) {
      router.push('/onboarding');
    } else if (isPending) {
      // Show success message for pending registration
      error.value = 'Registration submitted for approval. You will be notified when approved.';
      form.username = '';
      form.password = '';
      form.confirmPassword = '';
      // Don't redirect
    } else {
      router.push('/');
    }
  } catch (err) {
    error.value = err.message || 'An error occurred during registration';
  } finally {
    loading.value = false;
  }
}
</script>
