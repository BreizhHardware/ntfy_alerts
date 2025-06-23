export default defineNuxtRouteMiddleware(to => {
  const { auth } = useNuxtApp().$middleware || {};
  if (auth) {
    return auth(to);
  }
});

