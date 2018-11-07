export function get() {
  return localStorage.getItem('token');
}

export function set(token) {
  return localStorage.setItem('token', token);  
}