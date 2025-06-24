<template>
  <header class="py-6 bg-emerald-950 shadow-lg rounded-b-lg mb-4">
    <div class="container mx-auto px-4 flex justify-between items-center">
      <h1 class="text-4xl font-bold tracking-wide text-white">Github Ntfy</h1>

      <div v-if="auth.isAuthenticated" class="flex space-x-3">
        <UButton
          v-if="auth.isAdmin"
          to="/onboarding?force=true"
          variant="ghost"
          color="white"
          icon="i-heroicons-cog-8-tooth"
          size="sm"
        >
          Onboarding
        </UButton>

        <UButton
          to="/settings"
          variant="ghost"
          color="white"
          icon="i-heroicons-cog-6-tooth"
          size="sm"
        >
          Settings
        </UButton>

        <UButton
          @click="handleLogout"
          variant="ghost"
          color="white"
          icon="i-heroicons-arrow-right-on-rectangle"
          size="sm"
        >
          Logout
        </UButton>
      </div>
    </div>
  </header>
</template>

<script setup>
const auth = useAuth();
const router = useRouter();

const handleLogout = async () => {
  await auth.logout();
  router.push('/login');
};
</script>
