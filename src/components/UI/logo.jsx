import { chakra } from "@chakra-ui/react";
import NextLink from "next/link";

const Logo = () => {
  return (
    <NextLink href="/" passHref>
      <chakra.a fontSize="2rem" fontWeight="700">
        MEDICARE
      </chakra.a>
    </NextLink>
  );
};

export default Logo;
