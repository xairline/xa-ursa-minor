import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import {Badge, Button, Card, Container, Nav, Navbar} from "react-bootstrap";
import VibrationProfile from "./components/VibrationProfile.tsx";

function App() {
  const [serialNumber, setSerialNumber] = useState("");

  async function getSerialNumber() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    let res = await invoke("get_sn", {})
    console.log(res);
    setSerialNumber(res as string);
  }

  // State for storing "light" or "dark"
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    const interval = setInterval(() => {
      getSerialNumber();
    }, 1000); // Run every 5 seconds

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
      clearInterval(interval)
    };
  }, []);

  // Choose navbar variant based on theme
  const navbarVariant = theme === 'dark' ? 'dark' : 'light';

  const [selectedMenu, setSelectedMenu] = useState('option1'); // Default menu option

  const menuOptions = [
    {key: 'option1', label: 'Gear Touch Down', content: <VibrationProfile name={"Gear Touch Down"}/>},
    {key: 'option2', label: 'Taxi', content: <VibrationProfile name={"Taxi"}/>},
    {key: 'option3', label: 'Gear up/down', content: <VibrationProfile name={"Gear up/down"}/>},
    {key: 'option4', label: 'Elevator', content: <VibrationProfile name={"Elevator"}/>},
    {key: 'option5', label: 'Rudder', content: <VibrationProfile name={"Rudder"}/>},
    {key: 'option6', label: 'G Force - X', content: <VibrationProfile name={"G Force - X"}/>},
    {key: 'option7', label: 'G Force - Y', content: <VibrationProfile name={"G Force - Y"}/>},
    {key: 'option8', label: 'G Force - Z', content: <VibrationProfile name={"G Force - Z"}/>},
    {key: 'option9', label: 'Speed Brakes', content: <VibrationProfile name={"Speed Brakes"}/>},
  ];

  const renderContent = () => {
    const selectedOption = menuOptions.find(option => option.key === selectedMenu);
    return <>{selectedOption?.content}</>;
  };

  return (
    <div className="light-theme" style={{minHeight: '100vh'}}>
      {/* Sticky Navbar */}
      <Navbar sticky="top" expand="lg" variant={navbarVariant} bg={navbarVariant}>
        <Container fluid>
          <Navbar.Brand href="#home">
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

      {/* Main Layout: Left Menu + Main Content */}
      <div className="d-flex" style={{height: 'calc(100vh - 56px)'}}>
        {/* Left Menu */}
        <div
          className="bg-light border-end d-flex flex-column"
          style={{
            width: '340px',
            position: 'sticky',
            top: '56px',
            height: 'calc(100vh - 56px)',
          }}
        >
          {/* Menu Items */}
          <Navbar className="nav flex-column p-3 align-items-start bg-light rounded">
            <h5 className="mb-4">Vibration Profiles</h5>
            {menuOptions.map(option => (
              <Nav.Link
                key={option.key}
                className={`mb-2 btn btn-outline-info text-start ${
                  selectedMenu === option.key ? "active" : ""
                }`}
                onClick={() => setSelectedMenu(option.key)}
                style={{width: "300px"}}
              >
                {option.label}
              </Nav.Link>
            ))}
          </Navbar>

          {/* Spacer */}
          <div className="flex-grow-1"></div>

          {/* Card at the Bottom */}
          <div className="p-3">
            <Card className="p-3">
              <Card.Body className="d-flex flex-column align-items-center">
                <Card.Title><h2>URSA Minor</h2></Card.Title>
                <Card.Text>
                  <p>
                    Connection Status:
                    {
                      serialNumber.length > 0 ?
                        <Badge bg="success" style={{marginLeft: "12px"}}>Success</Badge> :
                        <Badge bg="danger" style={{marginLeft: "12px"}}>Error</Badge>
                    }
                  </p>
                  <small className="text-muted">Serial Number: {serialNumber}</small>
                </Card.Text>
                <Button variant="danger">Restart</Button>
              </Card.Body>
            </Card>
          </div>
        </div>

        {/* Main Content */}
        <div className="flex-grow-1 p-4">
          {renderContent()}
        </div>
      </div>
    </div>
  );
}

export default App;