import React from 'react'
import {gql, useQuery} from '@apollo/client'
import {useAuth} from '../hooks/useAuth'

const PING = gql`
  query Ping {
    ping
  }
`;

export const usePing = () => {
    return useQuery<string>(PING)
}
export const GraphQLPage = () => {
    const auth = useAuth()
    const pong = usePing()

    console.log('Response from server:', pong.loading ? 'loading' : pong.data)

    return (
        <div style={{height: '100%', fontSize: '1rem', textAlign: 'left'}}>
            <h1>GraphQL</h1>

            <h4>Query:</h4>
            <pre>
                {`query Ping {\n  ping\n}`}
            </pre>
            <h4>Query response:</h4>
            <pre>
                {!auth.isAuthenticated && 'Please login to test the GraphQL query.'}
                {auth.isAuthenticated && (pong.loading ? 'Executing GraphQL query...' : JSON.stringify(pong.data, null, 2))}
            </pre>

            <p><a href={"/graphql"}>Visit Playground</a></p>
        </div>
    )
}
