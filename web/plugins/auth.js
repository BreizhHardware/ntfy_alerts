// Authentication verification plugin
export default defineNuxtPlugin(() => {
  console.log('Authentication plugin loaded');

  addRouteMiddleware('auth', (to) => {
    console.log('Auth middleware executed for route:', to.path);

    if (to.path === '/login' || to.path === '/onboarding') {
      return;
    }

    if (process.client) {
      const token = localStorage.getItem('token');
      const user = localStorage.getItem('user');

      console.log('Authentication check:', !!token, !!user);

      if (!token || !user) {
        console.log('Redirecting to /login');
        return navigateTo('/login');
      }
    }
  }, { global: true });
});
