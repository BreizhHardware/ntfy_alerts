export default defineNuxtRouteMiddleware((to) => {
  if (to.path === '/login' || to.path === '/register') {
    return;
  }

  if (process.client) {
    const token = localStorage.getItem('token');
    const user = localStorage.getItem('user');

    if (!token || !user) {
      return navigateTo('/login');
    }
  }
});
