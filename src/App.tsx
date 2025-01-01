import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button, Container, Form, Nav, Navbar, Row } from "react-bootstrap";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  // State for storing "light" or "dark"
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    // Create a media query to detect dark mode
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)');

    // Handler for any changes in the OS preference
    const handleChange = (e: MediaQueryListEvent) => {
      setTheme(e.matches ? 'dark' : 'light');
    };

    // Initialize the theme on mount
    setTheme(prefersDark.matches ? 'dark' : 'light');

    // Listen for changes to the system theme
    prefersDark.addEventListener('change', handleChange);

    // Cleanup listener on unmount
    return () => {
      prefersDark.removeEventListener('change', handleChange);
    };
  }, []);

  // Choose navbar variant based on theme
  const navbarVariant = theme === 'dark' ? 'dark' : 'light';

  return (
    <div className={theme === 'dark' ? 'dark-theme' : 'light-theme'} style={{ minHeight: '100vh' }}>
      {/* Sticky Navbar */}
      <Navbar
        sticky="top"
        expand="lg"
        variant={navbarVariant}
        bg={navbarVariant}
      >
        <Container fluid>
          {/* 1. BRAND WITH ICON */}
          <Navbar.Brand href="#home">
            {/* Example icon—could be an SVG or an image */}
            <img
              src="/icon.png"
              width="30"
              height="30"
              className="d-inline-block align-top me-2"
              alt="App Logo"
            />
            XA URSA Minor Vibration Config
          </Navbar.Brand>
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