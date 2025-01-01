import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button, Container, Nav, Navbar } from "react-bootstrap";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  // Track the theme in React state
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  // Load theme from localStorage on initial mount (optional)
  useEffect(() => {
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
    if (savedTheme) {
      setTheme(savedTheme);
    }
  }, []);

  // Whenever theme changes, save it to localStorage (optional)
  useEffect(() => {
    localStorage.setItem('theme', theme);
  }, [theme]);

  // Toggle theme handler
  const toggleTheme = () => {
    setTheme((prev) => (prev === 'light' ? 'dark' : 'light'));
  };

  // Dynamically set class on container so it uses the correct CSS variables
  const appThemeClass = theme === 'light' ? 'light-theme' : 'dark-theme';
  // For the navbar specifically, you could add a separate class or rely on the theme
  const navbarClass = theme === 'light' ? 'navbar-light-theme' : 'navbar-dark-theme';

  return (
    <div className={appThemeClass} style={{ minHeight: '100vh' }}>
      {/* Sticky Navbar */}
      <Navbar
        className={navbarClass}
        sticky="top"
        expand="lg"
        variant={theme === 'light' ? 'light' : 'dark'}
      >
        <Container>
          <Navbar.Brand href="#home">My Tauri App</Navbar.Brand>
          <Nav className="ms-auto">
            <Button variant="outline-secondary" onClick={toggleTheme}>
              {theme === 'light' ? 'Switch to Dark' : 'Switch to Light'}
            </Button>
          </Nav>
        </Container>
      </Navbar>

      {/* Main content area */}
      <Container className="py-5">
        <h1>Hello from Tauri + React + TypeScript!</h1>
        <p>
          This is a simple layout. The navbar is sticky at the top.
          Click the button in the nav to toggle between light and dark themes.
        </p>
      </Container>
    </div>
  );
}

export default App;