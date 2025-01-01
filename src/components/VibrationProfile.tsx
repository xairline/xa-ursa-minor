import {Button, Card} from "react-bootstrap";

interface VibrationProfileProps {
  name: string;
}


const VibrationProfile = (props: VibrationProfileProps) => {
  return (
    <div className=" d-flex flex-column">
      <h1>PROFILE - {props.name}</h1>
      <div className="p-3">
        <Card className="p-3">
          <Card.Body className="d-flex flex-column align-items-center">
            <Card.Title><h2>Profile Chart</h2></Card.Title>
            <Card.Img variant="top" src="/icon.png"/>
          </Card.Body>
        </Card>
      </div>
      <div className="p-3">
        <Card className="p-3">
          <Card.Body className="d-flex flex-column align-items-center">
            <Card.Title><h2>Profile Parameters</h2></Card.Title>
            <Card.Text>
              <p>
                TODO
              </p>
            </Card.Text>
          </Card.Body>
        </Card>
      </div>
      <div className="p-3">
        <Card className="p-3">
          <Card.Body className="d-flex justify-content-evenly">
            <Button variant="primary">Try it!</Button>
            <Button variant="secondary">Save</Button>
          </Card.Body>
        </Card>
      </div>
    </div>
  );
};

export default VibrationProfile;
