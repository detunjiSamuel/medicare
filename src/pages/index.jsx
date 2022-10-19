import {
  Box,
  Button,
  Heading,
  Text,
  useColorModeValue,
} from "@chakra-ui/react";
import { NextSeo } from "next-seo";
import NextImage from "next/image";

import { seo, data } from "config";

const Home = () => {
  const color = useColorModeValue("telegram.500", "telegram.400");

  const isOdd = (num) => num % 2;

  const title = "Home";
  const description = seo.description;

  return (
    <>
      <NextSeo
        title={title}
        description={description}
        canonical={seo.canonical}
        openGraph={{
          title,
          description,
          images: [
            {
              url: `${seo.canonical}medicare.svg`,
              width: "350px",
              height: "350px",
              alt: "avatar bigheads",
            },
          ],
        }}
      />

      <Box
        as="section"
        d="flex"
        alignItems="center"
        flexDir="column"
        textAlign="center"
        py="4"
      >
        <NextImage
          src="/medicare.svg"
          width="750"
          height="750"
          alt="avatar bigheads"
          placeholder="blur"
          blurDataURL="L5I~of#i004mgjw]-4XA00?wL#xu"
          priority
        />
        <Box>
          <Heading as="h1" fontSize="2xl" fontWeight="500" py="2">
           Medicare{" "}
            <span role="img" aria-label="hand">
              👋🏻
            </span>
          </Heading>
          <Heading fontSize={["3xl", "4xl"]} fontWeight="700">
            <Text as="span" color={color}>
              Building
            </Text>{" "}
            a Smart Medical Ecosystem.
          </Heading>
          <Text py="4">
            A{" "}
            <Text as="span" fontWeight="600">
              decenteralized healthcare system
            </Text>{" "}
            and{" "}
            <Text as="span" fontWeight="600">
          our aim
            </Text>{" "}
             is to put the patient in control of their medical data, giving them the power to share the single, most comprehensive version of their record, with every organisation in their medical network.
          </Text>
          <Button
            colorScheme="telegram"
            variant="ghost"
            size="lg"
            fontSize="20px"
          >
            Get in touch
          </Button>
        </Box>
      </Box>

      <Box
        as="section"
        d="flex"
        alignItems="center"
        flexDir="column"
        textAlign={{ base: "center", lg: "left" }}
        py="4"
      >
        {data.map((item, index) => (
          <Box
            d={{ lg: "flex" }}
            justifyContent={{ lg: "center" }}
            alignItems={{ lg: "center" }}
            flexDir={{ lg: isOdd(index) == 1 && "row-reverse" }}
            key={index}
          >
            <Box
              w={{ base: "80%", lg: "35%" }}
              mx={{ base: "auto", lg: "0" }}
              pl={{ lg: isOdd(index) == 1 && "10" }}
              pr={{ lg: isOdd(index) == 0 && "10" }}
            >
              <NextImage
                src={item.image}
                width="500"
                height="500"
                alt={item.title}
                placeholder="blur"
                blurDataURL="L8LE.{~60000_3V@ITx^00t:V?-P"
              />
            </Box>

            <Box w={{ lg: "50%" }}>
              <Heading as="h1">{item.title}</Heading>
              <Text py="4">{item.description}</Text>
            </Box>
          </Box>
        ))}
      </Box>
    </>
  );
};

export default Home;
