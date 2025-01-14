import {Badge, Button, Card} from "react-bootstrap";
import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";


const UrsaMinorInfo = () => {
  const [serialNumber, setSerialNumber] = useState("");

  async function getSerialNumber() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    let res = await invoke("get_sn", {})
    setSerialNumber(res as string);
  }

  async function restartUrsaMinor() {
    let res = await invoke("restart_ursa_minor", {})
    console.log(res);
  }

  async function testUrsaMinor() {
    let res = await invoke("test_ursa_minor", {})
    console.log(res);
  }

  async function lightsOff() {
    let res = await invoke("lights_off", {})
    console.log(res);
  }

  async function lightsOn() {
    let res = await invoke("lights_on", {})
    console.log(res);
  }

  useEffect(() => {
    const interval = setInterval(() => {
      getSerialNumber();
    }, 1000); // Run every 5 seconds
    return () => clearInterval(interval);
  }, []);
  return (
    <div className="p-3">
      <Card className="p-3">
        <Card.Body className="d-flex flex-column align-items-center">
          <Card.Title>
            <h2>URSA Minor</h2>
          </Card.Title>
          <Card.Text className="border-top py-3">
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
          <div className="d-flex justify-content-evenly" style={{width: "100%", paddingTop: "20px"}}>
            <Button variant="primary" onClick={lightsOn}>Lights On</Button>
            <Button variant="dark" onClick={lightsOff}>Lights Off</Button>
          </div>
          <div className="d-flex justify-content-evenly" style={{width: "100%", paddingTop: "20px"}}>
            <Button variant="primary" onClick={testUrsaMinor}>Test</Button>
            <Button variant="danger" onClick={restartUrsaMinor}>Restart</Button>
          </div>
        </Card.Body>
      </Card>
    </div>
  )
    ;
};

export default UrsaMinorInfo;
