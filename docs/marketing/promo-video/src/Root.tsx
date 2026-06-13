import "./index.css";
import { Composition } from "remotion";
import { AgentIslandPromo } from "./AgentIslandPromo";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="AgentIslandPromo"
        component={AgentIslandPromo}
        durationInFrames={900}
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};
