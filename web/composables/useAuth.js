// Composable for managing authentication
export const useAuth = () => {
  const user = useState('user', () => null);
  const token = useState('token', () => null);
  const isFirstLogin = useState('isFirstLogin', () => false);

  // Initialize authentication state from localStorage
  onMounted(() => {
    const storedToken = localStorage.getItem('token');
    const storedUser = localStorage.getItem('user');

    if (storedToken && storedUser) {
      token.value = storedToken;
      user.value = JSON.parse(storedUser);
    }
  });

  // Login function
  const login = async (username, password) => {
    try {
      const response = await fetch('/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Login failed');
      }

      const data = await response.json();

      if (!data.success || !data.data) {
        throw new Error(data.message || 'Login failed');
      }

      // Store authentication information
      token.value = data.data.token;
      user.value = data.data.user;

      localStorage.setItem('token', data.data.token);
      localStorage.setItem('user', JSON.stringify(data.data.user));

      // Check if this is the first login
      const configResponse = await fetch('/is_configured');
      if (configResponse.ok) {
        const configData = await configResponse.json();
        isFirstLogin.value = !configData.data.settings_exist;
      }

      return data;
    } catch (error) {
      console.error('Login error:', error);
      throw error;
    }
  };

  // Registration function
  const register = async (username, password, isAdmin = false, isPending = false) => {
    try {
      const response = await fetch('/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username,
          password,
          is_admin: isAdmin,
          is_pending: isPending
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Registration failed');
      }

      const data = await response.json();

      if (!data.success || !data.data) {
        throw new Error(data.message || 'Registration failed');
      }

      // If registration is pending, don't store auth info
      if (isPending) {
        return data;
      }

      // Store authentication information
      token.value = data.data.token;
      user.value = data.data.user;

      localStorage.setItem('token', data.data.token);
      localStorage.setItem('user', JSON.stringify(data.data.user));

      // By default, consider a new registration needs onboarding
      isFirstLogin.value = true;

      return data;
    } catch (error) {
      console.error('Registration error:', error);
      throw error;
    }
  };

  // Logout function
  const logout = async () => {
    try {
      if (token.value) {
        await fetch('/auth/logout', {
          method: 'POST',
          headers: {
            'Authorization': token.value,
          },
        });
      }
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      // Clean up local authentication data
      token.value = null;
      user.value = null;
      localStorage.removeItem('token');
      localStorage.removeItem('user');
    }
  };

  // Check if user is authenticated
  const isAuthenticated = computed(() => !!token.value && !!user.value);

  // Check if user is admin
  const isAdmin = computed(() => isAuthenticated.value && user.value?.is_admin);

  // Get token for authenticated requests
  const getAuthHeader = () => {
    return token.value ? { Authorization: token.value } : {};
  };

  return {
    user,
    token,
    isFirstLogin,
    login,
    register,
    logout,
    isAuthenticated,
    isAdmin,
    getAuthHeader,
  };
};
